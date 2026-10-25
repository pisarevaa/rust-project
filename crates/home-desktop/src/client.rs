use std::fmt;
use std::time::Duration;

use home_api::{
    Device, DeviceId, ErrorCode, ErrorResponse, HouseReport, NewDevice, NewRoom, Room, RoomId,
    RoomSummary,
};
use reqwest::{Client, RequestBuilder, Response, Url};
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Переменная окружения с адресом backend-сервиса.
pub const BACKEND_URL_ENV: &str = "SMART_HOME_BACKEND_URL";
pub const DEFAULT_BACKEND_URL: &str = "http://127.0.0.1:3000";

const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Категория ошибки — по ней интерфейс решает, как показать ошибку.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// Неверная конфигурация клиента.
    Config,
    /// Backend не отвечает: не запущен, недоступен по сети или завис.
    Unavailable,
    NotFound,
    Validation,
    Conflict,
    /// Backend ответил ошибкой сервера или непонятным ответом.
    Server,
}

/// Ошибка запроса к backend. Уходит в интерфейс как `{ kind, message }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClientError {
    pub kind: ErrorKind,
    pub message: String,
}

impl ClientError {
    fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ClientError {}

/// HTTP-клиент REST API умного дома.
#[derive(Debug, Clone)]
pub struct BackendClient {
    base_url: Url,
    http: Client,
}

impl BackendClient {
    /// # Errors
    /// [`ErrorKind::Config`], если адрес некорректен или клиент не создался.
    pub fn new(base_url: &str) -> Result<Self, ClientError> {
        let base_url = Url::parse(base_url)
            .ok()
            .filter(|url| matches!(url.scheme(), "http" | "https") && url.has_host())
            .ok_or_else(|| {
                ClientError::new(
                    ErrorKind::Config,
                    format!("некорректный адрес backend «{base_url}», ожидается http://хост:порт"),
                )
            })?;
        // Адрес backend задан явно, поэтому системный прокси
        // (например, из `http_proxy`) не используем.
        let http = Client::builder()
            .no_proxy()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|e| {
                ClientError::new(
                    ErrorKind::Config,
                    format!("не удалось создать HTTP-клиент: {e}"),
                )
            })?;
        Ok(Self { base_url, http })
    }

    /// Клиент для адреса из [`BACKEND_URL_ENV`] или [`DEFAULT_BACKEND_URL`].
    ///
    /// # Errors
    /// См. [`BackendClient::new`].
    pub fn from_env() -> Result<Self, ClientError> {
        let url = std::env::var(BACKEND_URL_ENV).unwrap_or_else(|_| DEFAULT_BACKEND_URL.into());
        Self::new(&url)
    }

    #[must_use]
    pub fn base_url(&self) -> &str {
        self.base_url.as_str().trim_end_matches('/')
    }

    /// # Errors
    /// Ошибка сети или ответа backend.
    pub async fn rooms(&self) -> Result<Vec<RoomSummary>, ClientError> {
        self.fetch(self.http.get(self.url("/api/rooms"))).await
    }

    /// # Errors
    /// Ошибка сети или ответа backend.
    pub async fn room(&self, id: RoomId) -> Result<Room, ClientError> {
        self.fetch(self.http.get(self.url(&format!("/api/rooms/{id}"))))
            .await
    }

    /// # Errors
    /// Ошибка сети или ответа backend.
    pub async fn add_room(&self, new: &NewRoom) -> Result<Room, ClientError> {
        self.fetch(self.http.post(self.url("/api/rooms")).json(new))
            .await
    }

    /// # Errors
    /// Ошибка сети или ответа backend.
    pub async fn remove_room(&self, id: RoomId) -> Result<(), ClientError> {
        self.execute(self.http.delete(self.url(&format!("/api/rooms/{id}"))))
            .await
            .map(drop)
    }

    /// # Errors
    /// Ошибка сети или ответа backend.
    pub async fn devices(&self, room_id: RoomId) -> Result<Vec<Device>, ClientError> {
        self.fetch(
            self.http
                .get(self.url(&format!("/api/rooms/{room_id}/devices"))),
        )
        .await
    }

    /// # Errors
    /// Ошибка сети или ответа backend.
    pub async fn device(
        &self,
        room_id: RoomId,
        device_id: DeviceId,
    ) -> Result<Device, ClientError> {
        self.fetch(
            self.http
                .get(self.url(&format!("/api/rooms/{room_id}/devices/{device_id}"))),
        )
        .await
    }

    /// # Errors
    /// Ошибка сети или ответа backend.
    pub async fn add_device(
        &self,
        room_id: RoomId,
        new: &NewDevice,
    ) -> Result<Device, ClientError> {
        self.fetch(
            self.http
                .post(self.url(&format!("/api/rooms/{room_id}/devices")))
                .json(new),
        )
        .await
    }

    /// # Errors
    /// Ошибка сети или ответа backend.
    pub async fn remove_device(
        &self,
        room_id: RoomId,
        device_id: DeviceId,
    ) -> Result<(), ClientError> {
        self.execute(
            self.http
                .delete(self.url(&format!("/api/rooms/{room_id}/devices/{device_id}"))),
        )
        .await
        .map(drop)
    }

    /// # Errors
    /// Ошибка сети или ответа backend.
    pub async fn report(&self) -> Result<HouseReport, ClientError> {
        self.fetch(self.http.get(self.url("/api/report"))).await
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url())
    }

    async fn fetch<T: DeserializeOwned>(&self, request: RequestBuilder) -> Result<T, ClientError> {
        self.execute(request).await?.json().await.map_err(|e| {
            ClientError::new(
                ErrorKind::Server,
                format!("backend прислал некорректный ответ: {e}"),
            )
        })
    }

    /// Отправляет запрос; неуспешный HTTP-статус превращает в [`ClientError`].
    async fn execute(&self, request: RequestBuilder) -> Result<Response, ClientError> {
        let response = request.send().await.map_err(|e| self.network_error(&e))?;
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let error = match response.json::<ErrorResponse>().await {
            Ok(body) => ClientError::new(
                match body.code {
                    ErrorCode::NotFound => ErrorKind::NotFound,
                    ErrorCode::Validation => ErrorKind::Validation,
                    ErrorCode::Conflict => ErrorKind::Conflict,
                    ErrorCode::Internal => ErrorKind::Server,
                },
                body.message,
            ),
            Err(_) => ClientError::new(
                ErrorKind::Server,
                format!("backend ответил статусом {status}"),
            ),
        };
        Err(error)
    }

    fn network_error(&self, error: &reqwest::Error) -> ClientError {
        let base = self.base_url();
        let message = if error.is_timeout() {
            format!("backend {base} не ответил вовремя")
        } else if error.is_connect() {
            format!("backend недоступен по адресу {base} — проверьте, что сервис запущен")
        } else {
            format!("ошибка связи с backend {base}: {error}")
        };
        ClientError::new(ErrorKind::Unavailable, message)
    }
}

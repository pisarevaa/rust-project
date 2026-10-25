use std::fmt;

use axum::extract::rejection::{JsonRejection, PathRejection};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use home_api::{DeviceId, ErrorCode, ErrorResponse, RoomId};

/// Ошибка обработки запроса; превращается в JSON-ответ [`ErrorResponse`].
#[derive(Debug, PartialEq, Eq)]
pub enum ApiError {
    RoomNotFound(RoomId),
    DeviceNotFound {
        room_id: RoomId,
        device_id: DeviceId,
    },
    RouteNotFound,
    Validation(String),
    Conflict(String),
}

impl ApiError {
    #[must_use]
    pub fn status(&self) -> StatusCode {
        match self {
            Self::RoomNotFound(_) | Self::DeviceNotFound { .. } | Self::RouteNotFound => {
                StatusCode::NOT_FOUND
            }
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Conflict(_) => StatusCode::CONFLICT,
        }
    }

    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::RoomNotFound(_) | Self::DeviceNotFound { .. } | Self::RouteNotFound => {
                ErrorCode::NotFound
            }
            Self::Validation(_) => ErrorCode::Validation,
            Self::Conflict(_) => ErrorCode::Conflict,
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RoomNotFound(id) => write!(f, "комната с id {id} не найдена"),
            Self::DeviceNotFound { room_id, device_id } => write!(
                f,
                "устройство с id {device_id} не найдено в комнате с id {room_id}"
            ),
            Self::RouteNotFound => f.write_str("такого маршрута нет"),
            Self::Validation(details) => write!(f, "некорректные данные: {details}"),
            Self::Conflict(details) => f.write_str(details),
        }
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorResponse {
            code: self.code(),
            message: self.to_string(),
        };
        (self.status(), axum::Json(body)).into_response()
    }
}

// Ошибки разбора тела и пути тоже отдаем в едином JSON-формате,
// а не текстом по умолчанию из axum.
impl From<JsonRejection> for ApiError {
    fn from(rejection: JsonRejection) -> Self {
        Self::Validation(rejection.body_text())
    }
}

impl From<PathRejection> for ApiError {
    fn from(rejection: PathRejection) -> Self {
        Self::Validation(rejection.body_text())
    }
}

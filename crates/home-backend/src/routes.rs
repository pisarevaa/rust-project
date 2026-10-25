use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use axum::extract::{FromRequest, FromRequestParts, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use home_api::{Device, DeviceId, HouseReport, NewDevice, NewRoom, Room, RoomId, RoomSummary};
use serde::Serialize;

use crate::error::ApiError;
use crate::store::Store;

#[derive(Clone, Default)]
struct AppState {
    store: Arc<RwLock<Store>>,
}

impl AppState {
    // Обработчики не паникуют под блокировкой, но если это все же случится,
    // данные в хранилище остаются согласованными — продолжаем работать.
    fn read(&self) -> RwLockReadGuard<'_, Store> {
        self.store.read().unwrap_or_else(PoisonError::into_inner)
    }

    fn write(&self) -> RwLockWriteGuard<'_, Store> {
        self.store.write().unwrap_or_else(PoisonError::into_inner)
    }
}

/// `Json` с ошибками разбора в формате [`ApiError`].
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
struct Json<T>(T);

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}

/// `Path` с ошибками разбора в формате [`ApiError`].
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ApiError))]
struct Path<T>(T);

/// Маршрутизатор REST API с пустым домом.
pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/rooms", get(list_rooms).post(create_room))
        .route("/api/rooms/{room_id}", get(get_room).delete(delete_room))
        .route(
            "/api/rooms/{room_id}/devices",
            get(list_devices).post(create_device),
        )
        .route(
            "/api/rooms/{room_id}/devices/{device_id}",
            get(get_device).delete(delete_device),
        )
        .route("/api/report", get(report))
        .fallback(|| async { ApiError::RouteNotFound })
        .with_state(AppState::default())
}

async fn health() -> &'static str {
    "ok"
}

async fn list_rooms(State(state): State<AppState>) -> Json<Vec<RoomSummary>> {
    Json(state.read().rooms())
}

async fn create_room(
    State(state): State<AppState>,
    Json(new): Json<NewRoom>,
) -> Result<(StatusCode, Json<Room>), ApiError> {
    let room = state.write().add_room(&new)?;
    Ok((StatusCode::CREATED, Json(room)))
}

async fn get_room(
    State(state): State<AppState>,
    Path(room_id): Path<RoomId>,
) -> Result<Json<Room>, ApiError> {
    state.read().room(room_id).map(Json)
}

async fn delete_room(
    State(state): State<AppState>,
    Path(room_id): Path<RoomId>,
) -> Result<StatusCode, ApiError> {
    state.write().remove_room(room_id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_devices(
    State(state): State<AppState>,
    Path(room_id): Path<RoomId>,
) -> Result<Json<Vec<Device>>, ApiError> {
    state.read().devices(room_id).map(Json)
}

async fn create_device(
    State(state): State<AppState>,
    Path(room_id): Path<RoomId>,
    Json(new): Json<NewDevice>,
) -> Result<(StatusCode, Json<Device>), ApiError> {
    let device = state.write().add_device(room_id, &new)?;
    Ok((StatusCode::CREATED, Json(device)))
}

async fn get_device(
    State(state): State<AppState>,
    Path((room_id, device_id)): Path<(RoomId, DeviceId)>,
) -> Result<Json<Device>, ApiError> {
    state.read().device(room_id, device_id).map(Json)
}

async fn delete_device(
    State(state): State<AppState>,
    Path((room_id, device_id)): Path<(RoomId, DeviceId)>,
) -> Result<StatusCode, ApiError> {
    state.write().remove_device(room_id, device_id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn report(State(state): State<AppState>) -> Json<HouseReport> {
    Json(state.read().report())
}

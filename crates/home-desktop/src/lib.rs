//! Desktop-клиент умного дома. Интерфейс вызывает команды Tauri через
//! `invoke`, а команды обращаются к backend-сервису по HTTP.

pub mod client;

pub use client::{BackendClient, ClientError, ErrorKind};
use home_api::{
    Device, DeviceId, DeviceKind, DeviceStatus, HouseReport, NewDevice, NewRoom, Room, RoomId,
    RoomSummary,
};
use tauri::State;

// Tauri передает состояние в команды только по значению.
#[allow(clippy::needless_pass_by_value)]
#[tauri::command]
fn backend_url(client: State<'_, BackendClient>) -> String {
    client.base_url().to_owned()
}

#[tauri::command]
async fn list_rooms(client: State<'_, BackendClient>) -> Result<Vec<RoomSummary>, ClientError> {
    client.rooms().await
}

#[tauri::command]
async fn get_room(client: State<'_, BackendClient>, room_id: RoomId) -> Result<Room, ClientError> {
    client.room(room_id).await
}

#[tauri::command]
async fn create_room(client: State<'_, BackendClient>, name: String) -> Result<Room, ClientError> {
    client.add_room(&NewRoom { name }).await
}

#[tauri::command]
async fn delete_room(client: State<'_, BackendClient>, room_id: RoomId) -> Result<(), ClientError> {
    client.remove_room(room_id).await
}

#[tauri::command]
async fn list_devices(
    client: State<'_, BackendClient>,
    room_id: RoomId,
) -> Result<Vec<Device>, ClientError> {
    client.devices(room_id).await
}

#[tauri::command]
async fn get_device(
    client: State<'_, BackendClient>,
    room_id: RoomId,
    device_id: DeviceId,
) -> Result<Device, ClientError> {
    client.device(room_id, device_id).await
}

#[tauri::command]
async fn create_device(
    client: State<'_, BackendClient>,
    room_id: RoomId,
    name: String,
    kind: DeviceKind,
    status: DeviceStatus,
) -> Result<Device, ClientError> {
    client
        .add_device(room_id, &NewDevice { name, kind, status })
        .await
}

#[tauri::command]
async fn delete_device(
    client: State<'_, BackendClient>,
    room_id: RoomId,
    device_id: DeviceId,
) -> Result<(), ClientError> {
    client.remove_device(room_id, device_id).await
}

#[tauri::command]
async fn house_report(client: State<'_, BackendClient>) -> Result<HouseReport, ClientError> {
    client.report().await
}

/// Запускает окно приложения; возвращает управление после его закрытия.
///
/// # Errors
/// Ошибка инициализации Tauri.
pub fn run(client: BackendClient) -> tauri::Result<()> {
    tauri::Builder::default()
        .manage(client)
        .invoke_handler(tauri::generate_handler![
            backend_url,
            list_rooms,
            get_room,
            create_room,
            delete_room,
            list_devices,
            get_device,
            create_device,
            delete_device,
            house_report,
        ])
        .run(tauri::generate_context!())
}

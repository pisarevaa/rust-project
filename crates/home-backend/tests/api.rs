//! Интеграционные тесты: сервис запускается на свободном порту,
//! запросы идут по настоящему HTTP.

use home_api::{
    Device, DeviceKind, DeviceStatus, ErrorCode, ErrorResponse, HouseReport, NewDevice, NewRoom,
    Room, RoomSummary,
};
use reqwest::{Client, StatusCode};
use tokio::net::TcpListener;

struct TestServer {
    base: String,
    http: Client,
}

impl TestServer {
    async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(home_backend::serve(listener, std::future::pending()));
        Self {
            base: format!("http://{addr}"),
            http: Client::builder().no_proxy().build().unwrap(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    async fn create_room(&self, name: &str) -> Room {
        let response = self
            .http
            .post(self.url("/api/rooms"))
            .json(&NewRoom { name: name.into() })
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        response.json().await.unwrap()
    }

    async fn create_device(&self, room: &Room, name: &str, kind: DeviceKind) -> Device {
        let response = self
            .http
            .post(self.url(&format!("/api/rooms/{}/devices", room.id)))
            .json(&NewDevice {
                name: name.into(),
                kind,
                status: DeviceStatus::On,
            })
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        response.json().await.unwrap()
    }

    async fn get(&self, path: &str) -> reqwest::Response {
        self.http.get(self.url(path)).send().await.unwrap()
    }

    async fn delete(&self, path: &str) -> reqwest::Response {
        self.http.delete(self.url(path)).send().await.unwrap()
    }
}

async fn expect_error(response: reqwest::Response, status: StatusCode, code: ErrorCode) {
    assert_eq!(response.status(), status);
    let body: ErrorResponse = response.json().await.unwrap();
    assert_eq!(body.code, code);
    assert!(!body.message.is_empty());
}

#[tokio::test]
async fn create_and_get_room() {
    let server = TestServer::start().await;
    let created = server.create_room("Кухня").await;
    assert_eq!(created.name, "Кухня");
    assert!(created.devices.is_empty());

    let response = server.get(&format!("/api/rooms/{}", created.id)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let fetched: Room = response.json().await.unwrap();
    assert_eq!(fetched, created);
}

#[tokio::test]
async fn list_rooms() {
    let server = TestServer::start().await;
    let kitchen = server.create_room("Кухня").await;
    let bedroom = server.create_room("Спальня").await;
    server
        .create_device(&kitchen, "Чайник", DeviceKind::Socket)
        .await;

    let rooms: Vec<RoomSummary> = server.get("/api/rooms").await.json().await.unwrap();
    assert_eq!(
        rooms,
        vec![
            RoomSummary {
                id: kitchen.id,
                name: "Кухня".into(),
                device_count: 1,
            },
            RoomSummary {
                id: bedroom.id,
                name: "Спальня".into(),
                device_count: 0,
            },
        ]
    );
}

#[tokio::test]
async fn create_and_get_device() {
    let server = TestServer::start().await;
    let room = server.create_room("Кухня").await;
    let created = server
        .create_device(&room, "Термометр", DeviceKind::Thermometer)
        .await;
    assert_eq!(created.room_id, room.id);
    assert_eq!(created.kind, DeviceKind::Thermometer);
    assert_eq!(created.status, DeviceStatus::On);

    let response = server
        .get(&format!("/api/rooms/{}/devices/{}", room.id, created.id))
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    let fetched: Device = response.json().await.unwrap();
    assert_eq!(fetched, created);
}

#[tokio::test]
async fn list_room_devices() {
    let server = TestServer::start().await;
    let kitchen = server.create_room("Кухня").await;
    let bedroom = server.create_room("Спальня").await;
    let kettle = server
        .create_device(&kitchen, "Чайник", DeviceKind::Socket)
        .await;
    let lamp = server
        .create_device(&kitchen, "Лампа", DeviceKind::Light)
        .await;
    server
        .create_device(&bedroom, "Ночник", DeviceKind::Light)
        .await;

    let devices: Vec<Device> = server
        .get(&format!("/api/rooms/{}/devices", kitchen.id))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(devices, vec![kettle, lamp]);
}

#[tokio::test]
async fn delete_device_and_room() {
    let server = TestServer::start().await;
    let room = server.create_room("Кухня").await;
    let device = server
        .create_device(&room, "Чайник", DeviceKind::Socket)
        .await;
    let device_path = format!("/api/rooms/{}/devices/{}", room.id, device.id);
    let room_path = format!("/api/rooms/{}", room.id);

    assert_eq!(
        server.delete(&device_path).await.status(),
        StatusCode::NO_CONTENT
    );
    expect_error(
        server.get(&device_path).await,
        StatusCode::NOT_FOUND,
        ErrorCode::NotFound,
    )
    .await;

    assert_eq!(
        server.delete(&room_path).await.status(),
        StatusCode::NO_CONTENT
    );
    expect_error(
        server.get(&room_path).await,
        StatusCode::NOT_FOUND,
        ErrorCode::NotFound,
    )
    .await;
    let rooms: Vec<RoomSummary> = server.get("/api/rooms").await.json().await.unwrap();
    assert!(rooms.is_empty());
}

#[tokio::test]
async fn house_report() {
    let server = TestServer::start().await;
    let kitchen = server.create_room("Кухня").await;
    let bedroom = server.create_room("Спальня").await;
    server.create_room("Кладовка").await;
    server
        .create_device(&kitchen, "Чайник", DeviceKind::Socket)
        .await;
    server
        .create_device(&kitchen, "Лампа", DeviceKind::Light)
        .await;
    server
        .create_device(&bedroom, "Ночник", DeviceKind::Light)
        .await;

    let response = server.get("/api/report").await;
    assert_eq!(response.status(), StatusCode::OK);
    let report: HouseReport = response.json().await.unwrap();
    assert_eq!(report.room_count, 3);
    assert_eq!(report.device_count, 3);
    assert_eq!(report.devices_by_kind[&DeviceKind::Light], 2);
    assert_eq!(report.devices_by_kind[&DeviceKind::Socket], 1);
    assert_eq!(report.devices_by_kind[&DeviceKind::Thermometer], 0);
    assert_eq!(report.devices_by_status[&DeviceStatus::On], 3);
    let counts: Vec<(&str, usize)> = report
        .rooms
        .iter()
        .map(|room| (room.name.as_str(), room.device_count))
        .collect();
    assert_eq!(counts, [("Кухня", 2), ("Спальня", 1), ("Кладовка", 0)]);
}

#[tokio::test]
async fn missing_room_is_not_found() {
    let server = TestServer::start().await;
    expect_error(
        server.get("/api/rooms/42").await,
        StatusCode::NOT_FOUND,
        ErrorCode::NotFound,
    )
    .await;
    expect_error(
        server.get("/api/rooms/42/devices").await,
        StatusCode::NOT_FOUND,
        ErrorCode::NotFound,
    )
    .await;
}

#[tokio::test]
async fn device_cannot_be_added_to_missing_room() {
    let server = TestServer::start().await;
    let response = server
        .http
        .post(server.url("/api/rooms/42/devices"))
        .json(&NewDevice {
            name: "Чайник".into(),
            kind: DeviceKind::Socket,
            status: DeviceStatus::Off,
        })
        .send()
        .await
        .unwrap();
    expect_error(response, StatusCode::NOT_FOUND, ErrorCode::NotFound).await;
}

#[tokio::test]
async fn invalid_requests_are_rejected() {
    let server = TestServer::start().await;

    let empty_name = server
        .http
        .post(server.url("/api/rooms"))
        .json(&NewRoom { name: "  ".into() })
        .send()
        .await
        .unwrap();
    expect_error(
        empty_name,
        StatusCode::UNPROCESSABLE_ENTITY,
        ErrorCode::Validation,
    )
    .await;

    let room = server.create_room("Кухня").await;
    let unknown_kind = server
        .http
        .post(server.url(&format!("/api/rooms/{}/devices", room.id)))
        .json(&serde_json_value(r#"{"name": "Утюг", "kind": "iron"}"#))
        .send()
        .await
        .unwrap();
    expect_error(
        unknown_kind,
        StatusCode::UNPROCESSABLE_ENTITY,
        ErrorCode::Validation,
    )
    .await;

    expect_error(
        server.get("/api/rooms/abc").await,
        StatusCode::UNPROCESSABLE_ENTITY,
        ErrorCode::Validation,
    )
    .await;

    let duplicate = server
        .http
        .post(server.url("/api/rooms"))
        .json(&NewRoom {
            name: "кухня".into(),
        })
        .send()
        .await
        .unwrap();
    expect_error(duplicate, StatusCode::CONFLICT, ErrorCode::Conflict).await;
}

#[tokio::test]
async fn unknown_route_is_json_not_found() {
    let server = TestServer::start().await;
    expect_error(
        server.get("/api/nope").await,
        StatusCode::NOT_FOUND,
        ErrorCode::NotFound,
    )
    .await;
}

fn serde_json_value(json: &str) -> serde_json::Value {
    serde_json::from_str(json).unwrap()
}

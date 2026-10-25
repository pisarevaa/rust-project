//! Проверка HTTP-клиента, которым пользуются команды Tauri,
//! против настоящего backend-сервиса.

use home_api::{DeviceKind, DeviceStatus, NewDevice, NewRoom};
use home_desktop::{BackendClient, ErrorKind};
use tokio::net::TcpListener;

async fn start_backend() -> BackendClient {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(home_backend::serve(listener, std::future::pending()));
    BackendClient::new(&format!("http://{addr}/")).unwrap()
}

#[tokio::test]
async fn client_round_trip() {
    let client = start_backend().await;
    let room = client
        .add_room(&NewRoom {
            name: "Кухня".into(),
        })
        .await
        .unwrap();
    let device = client
        .add_device(
            room.id,
            &NewDevice {
                name: "Чайник".into(),
                kind: DeviceKind::Socket,
                status: DeviceStatus::On,
            },
        )
        .await
        .unwrap();

    assert_eq!(client.rooms().await.unwrap()[0].device_count, 1);
    let expected = std::slice::from_ref(&device);
    assert_eq!(client.room(room.id).await.unwrap().devices, expected);
    assert_eq!(client.devices(room.id).await.unwrap(), expected);
    assert_eq!(client.device(room.id, device.id).await.unwrap(), device);
    assert_eq!(client.report().await.unwrap().device_count, 1);

    client.remove_device(room.id, device.id).await.unwrap();
    client.remove_room(room.id).await.unwrap();
    assert!(client.rooms().await.unwrap().is_empty());
}

#[tokio::test]
async fn backend_errors_are_classified() {
    let client = start_backend().await;
    assert_eq!(client.room(7).await.unwrap_err().kind, ErrorKind::NotFound);
    let error = client
        .add_room(&NewRoom { name: " ".into() })
        .await
        .unwrap_err();
    assert_eq!(error.kind, ErrorKind::Validation);
    assert!(!error.message.is_empty());
}

#[tokio::test]
async fn unavailable_backend_is_reported() {
    // Занимаем порт и сразу освобождаем: на нем точно никто не слушает.
    let addr = TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap()
        .local_addr()
        .unwrap();
    let client = BackendClient::new(&format!("http://{addr}")).unwrap();
    let error = client.rooms().await.unwrap_err();
    assert_eq!(error.kind, ErrorKind::Unavailable);
}

#[test]
fn invalid_backend_url_is_rejected() {
    for url in ["", "localhost:3000", "ftp://example.com"] {
        assert_eq!(
            BackendClient::new(url).unwrap_err().kind,
            ErrorKind::Config,
            "{url}"
        );
    }
}

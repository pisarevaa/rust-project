//! REST backend-сервис умного дома. Данные хранятся в памяти процесса.

pub mod error;
pub mod routes;
pub mod store;

use std::future::Future;

pub use error::ApiError;
pub use routes::router;
pub use store::Store;
use tokio::net::TcpListener;

/// Переменная окружения с адресом, на котором слушает сервис.
pub const ADDR_ENV: &str = "SMART_HOME_ADDR";
pub const DEFAULT_ADDR: &str = "127.0.0.1:3000";

/// Обслуживает запросы на `listener`, пока не завершится `shutdown`.
///
/// # Errors
/// Ошибка ввода-вывода при работе с сокетом.
pub async fn serve(
    listener: TcpListener,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    axum::serve(listener, router())
        .with_graceful_shutdown(shutdown)
        .await
}

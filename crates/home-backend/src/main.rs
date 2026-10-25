use std::process::ExitCode;

use home_backend::{serve, ADDR_ENV, DEFAULT_ADDR};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> ExitCode {
    let addr = std::env::var(ADDR_ENV).unwrap_or_else(|_| DEFAULT_ADDR.to_owned());
    let listener = match TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Не удалось занять адрес {addr}: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("Backend умного дома слушает http://{addr}");

    let shutdown = async {
        // Если обработчик сигнала не установился, просто работаем до kill.
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
        println!("Завершение работы");
    };
    if let Err(e) = serve(listener, shutdown).await {
        eprintln!("Ошибка сервера: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

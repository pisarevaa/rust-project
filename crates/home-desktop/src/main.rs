// В release-сборке на Windows не открываем лишнее консольное окно.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::ExitCode;

use home_desktop::BackendClient;

fn main() -> ExitCode {
    let client = match BackendClient::from_env() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Ошибка конфигурации: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("Адрес backend: {}", client.base_url());
    if let Err(e) = home_desktop::run(client) {
        eprintln!("Не удалось запустить приложение: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

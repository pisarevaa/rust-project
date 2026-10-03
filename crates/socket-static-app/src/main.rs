//! Приложение, статически линкующее `libsmart_socket_ffi.a` и работающее
//! с розеткой только через ее Си ABI.
//!
//! Без аргументов использует имитацию розетки. С адресом
//! (`socket-static-app 127.0.0.1:8181`) обращается к имитатору по TCP.

use std::ffi::{c_char, CString};

#[repr(C)]
struct SmartSocket {
    _private: [u8; 0],
}

type SocketStatus = i32;
const SOCKET_STATUS_OK: SocketStatus = 0;

#[link(name = "smart_socket_ffi", kind = "static")]
extern "C" {
    fn smart_socket_new_mock(power: f64) -> *mut SmartSocket;
    fn smart_socket_new_tcp(address: *const c_char) -> *mut SmartSocket;
    fn smart_socket_free(socket: *mut SmartSocket);
    fn smart_socket_turn_on(socket: *const SmartSocket) -> SocketStatus;
    fn smart_socket_turn_off(socket: *const SmartSocket) -> SocketStatus;
    fn smart_socket_is_on(socket: *const SmartSocket, is_on: *mut bool) -> SocketStatus;
    fn smart_socket_power(socket: *const SmartSocket, power: *mut f64) -> SocketStatus;
}

fn main() {
    let socket = if let Some(address) = std::env::args().nth(1) {
        println!("Розетка по адресу {address}");
        let address = CString::new(address).expect("адрес без нулевых байтов");
        unsafe { smart_socket_new_tcp(address.as_ptr()) }
    } else {
        println!("Имитация розетки мощностью 1500 Вт");
        unsafe { smart_socket_new_mock(1500.0) }
    };
    assert!(!socket.is_null(), "библиотека не создала розетку");

    print_state(socket);
    println!("Включаем розетку");

    check(unsafe { smart_socket_turn_on(socket) });
    print_state(socket);
    println!("Выключаем розетку");

    check(unsafe { smart_socket_turn_off(socket) });
    print_state(socket);

    unsafe { smart_socket_free(socket) };
}

fn print_state(socket: *const SmartSocket) {
    let mut is_on = false;
    let mut power = 0.0;

    unsafe {
        check(smart_socket_is_on(socket, &raw mut is_on));
        check(smart_socket_power(socket, &raw mut power));
    }
    let state = if is_on {
        "включена"
    } else {
        "выключена"
    };
    println!("  розетка {state}, мощность {power:.1} Вт");
}

fn check(status: SocketStatus) {
    if status != SOCKET_STATUS_OK {
        eprintln!("Ошибка обращения к розетке, код {status}");
        std::process::exit(1);
    }
}

//! Приложение, загружающее динамическую библиотеку умной розетки в runtime
//! и работающее с розеткой только через ее Си ABI.
//!
//! Библиотека ищется рядом с исполняемым файлом (или в соседнем `deps`,
//! куда ее кладет cargo); другой путь можно задать
//! переменной окружения `SMART_SOCKET_LIB`. Без аргументов используется
//! имитация розетки, с адресом (`socket-dynamic-app 127.0.0.1:8181`) —
//! обращение к имитатору по TCP.

use libloading::{Library, Symbol};
use std::error::Error;
use std::ffi::{c_char, CString};
use std::path::PathBuf;

#[repr(C)]
struct SmartSocket {
    _private: [u8; 0],
}

type SocketStatus = i32;
const SOCKET_STATUS_OK: SocketStatus = 0;

type NewMockFn = unsafe extern "C" fn(f64) -> *mut SmartSocket;
type NewTcpFn = unsafe extern "C" fn(*const c_char) -> *mut SmartSocket;
type FreeFn = unsafe extern "C" fn(*mut SmartSocket);
type SwitchFn = unsafe extern "C" fn(*const SmartSocket) -> SocketStatus;
type IsOnFn = unsafe extern "C" fn(*const SmartSocket, *mut bool) -> SocketStatus;
type PowerFn = unsafe extern "C" fn(*const SmartSocket, *mut f64) -> SocketStatus;

struct SocketApi<'lib> {
    new_mock: Symbol<'lib, NewMockFn>,
    new_tcp: Symbol<'lib, NewTcpFn>,
    free: Symbol<'lib, FreeFn>,
    turn_on: Symbol<'lib, SwitchFn>,
    turn_off: Symbol<'lib, SwitchFn>,
    is_on: Symbol<'lib, IsOnFn>,
    power: Symbol<'lib, PowerFn>,
}

impl<'lib> SocketApi<'lib> {
    fn load(library: &'lib Library) -> Result<Self, libloading::Error> {
        unsafe {
            Ok(Self {
                new_mock: library.get(b"smart_socket_new_mock\0")?,
                new_tcp: library.get(b"smart_socket_new_tcp\0")?,
                free: library.get(b"smart_socket_free\0")?,
                turn_on: library.get(b"smart_socket_turn_on\0")?,
                turn_off: library.get(b"smart_socket_turn_off\0")?,
                is_on: library.get(b"smart_socket_is_on\0")?,
                power: library.get(b"smart_socket_power\0")?,
            })
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let path = library_path()?;
    println!("Загружаем {}", path.display());
    let library = unsafe { Library::new(&path) }
        .map_err(|e| format!("не удалось загрузить библиотеку: {e}"))?;
    let api = SocketApi::load(&library)?;

    let socket = if let Some(address) = std::env::args().nth(1) {
        println!("Розетка по адресу {address}");
        let address = CString::new(address)?;
        unsafe { (api.new_tcp)(address.as_ptr()) }
    } else {
        println!("Имитация розетки мощностью 2000 Вт");
        unsafe { (api.new_mock)(2000.0) }
    };
    if socket.is_null() {
        return Err("библиотека не создала розетку".into());
    }

    let result = demonstrate(&api, socket);
    unsafe { (api.free)(socket) };
    result
}

fn demonstrate(api: &SocketApi, socket: *const SmartSocket) -> Result<(), Box<dyn Error>> {
    print_state(api, socket)?;
    println!("Включаем розетку");
    check(unsafe { (api.turn_on)(socket) })?;
    print_state(api, socket)?;
    println!("Выключаем розетку");
    check(unsafe { (api.turn_off)(socket) })?;
    print_state(api, socket)
}

fn print_state(api: &SocketApi, socket: *const SmartSocket) -> Result<(), Box<dyn Error>> {
    let mut is_on = false;
    let mut power = 0.0;
    unsafe {
        check((api.is_on)(socket, &raw mut is_on))?;
        check((api.power)(socket, &raw mut power))?;
    }
    let state = if is_on {
        "включена"
    } else {
        "выключена"
    };
    println!("  розетка {state}, мощность {power:.1} Вт");
    Ok(())
}

fn check(status: SocketStatus) -> Result<(), Box<dyn Error>> {
    if status == SOCKET_STATUS_OK {
        Ok(())
    } else {
        Err(format!("ошибка обращения к розетке, код {status}").into())
    }
}

fn library_path() -> Result<PathBuf, Box<dyn Error>> {
    if let Some(path) = std::env::var_os("SMART_SOCKET_LIB") {
        return Ok(path.into());
    }
    let exe = std::env::current_exe()?;
    let dir = exe.parent().ok_or("у исполняемого файла нет каталога")?;
    let file_name = libloading::library_filename("smart_socket_ffi");
    let beside = dir.join(&file_name);
    if beside.exists() {
        Ok(beside)
    } else {
        Ok(dir.join("deps").join(file_name))
    }
}

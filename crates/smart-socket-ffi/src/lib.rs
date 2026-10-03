//! Умная розетка с Си ABI.
//!
//! Розетка передается через непрозрачный указатель [`SmartSocket`]:
//! создается функциями `smart_socket_new_*`, освобождается
//! [`smart_socket_free`]. Остальные функции возвращают [`SocketStatus`],
//! а результат запроса кладут по переданному указателю.
//! Заголовок для Си лежит в `include/smart_socket.h`.

use smart_home::error::DeviceError;
use std::ffi::{c_char, CStr};

pub use smart_home::socket::SmartSocket;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketStatus {
    Ok = 0,
    NullPointer = 1,
    ConnectionError = 2,
    Timeout = 3,
    ProtocolError = 4,
}

impl From<&DeviceError> for SocketStatus {
    fn from(e: &DeviceError) -> Self {
        match e {
            DeviceError::Io(_) | DeviceError::NoData => Self::ConnectionError,
            DeviceError::Timeout => Self::Timeout,
            DeviceError::Protocol(_) => Self::ProtocolError,
        }
    }
}

#[no_mangle]
pub extern "C" fn smart_socket_new_mock(power: f64) -> *mut SmartSocket {
    Box::into_raw(Box::new(SmartSocket::mock(power)))
}

/// Создает розетку, подключаемую по TCP к `address` (например, `"127.0.0.1:8080"`).
/// Возвращает null, если `address` равен null или не является корректной UTF-8 строкой.
///
/// # Safety
///
/// `address` должен быть null или указывать на корректную Си-строку,
/// завершенную нулем и действительную на время вызова.
#[no_mangle]
pub unsafe extern "C" fn smart_socket_new_tcp(address: *const c_char) -> *mut SmartSocket {
    if address.is_null() {
        return std::ptr::null_mut();
    }
    match unsafe { CStr::from_ptr(address) }.to_str() {
        Ok(address) => Box::into_raw(Box::new(SmartSocket::reconnecting(address))),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Освобождает розетку. Передача null допустима и ничего не делает.
///
/// # Safety
///
/// `socket` должен быть null или указателем, полученным из `smart_socket_new_*`
/// и еще не освобожденным. После вызова указатель использовать нельзя.
#[no_mangle]
pub unsafe extern "C" fn smart_socket_free(socket: *mut SmartSocket) {
    if !socket.is_null() {
        drop(unsafe { Box::from_raw(socket) });
    }
}

/// Включает розетку.
///
/// # Safety
///
/// `socket` должен быть null или действительным указателем из `smart_socket_new_*`.
#[no_mangle]
pub unsafe extern "C" fn smart_socket_turn_on(socket: *const SmartSocket) -> SocketStatus {
    with_socket(unsafe { socket.as_ref() }, SmartSocket::turn_on, |()| {})
}

/// Выключает розетку.
///
/// # Safety
///
/// `socket` должен быть null или действительным указателем из `smart_socket_new_*`.
#[no_mangle]
pub unsafe extern "C" fn smart_socket_turn_off(socket: *const SmartSocket) -> SocketStatus {
    with_socket(unsafe { socket.as_ref() }, SmartSocket::turn_off, |()| {})
}

/// Записывает в `is_on`, включена ли розетка.
///
/// # Safety
///
/// `socket` должен быть null или действительным указателем из `smart_socket_new_*`;
/// `is_on` должен быть null или указывать на память, доступную для записи `bool`.
#[no_mangle]
pub unsafe extern "C" fn smart_socket_is_on(
    socket: *const SmartSocket,
    is_on: *mut bool,
) -> SocketStatus {
    if is_on.is_null() {
        return SocketStatus::NullPointer;
    }
    with_socket(
        unsafe { socket.as_ref() },
        SmartSocket::is_on,
        |value| unsafe {
            is_on.write(value);
        },
    )
}

/// Записывает в `power` текущую потребляемую мощность.
///
/// # Safety
///
/// `socket` должен быть null или действительным указателем из `smart_socket_new_*`;
/// `power` должен быть null или указывать на память, доступную для записи `f64`.
#[no_mangle]
pub unsafe extern "C" fn smart_socket_power(
    socket: *const SmartSocket,
    power: *mut f64,
) -> SocketStatus {
    if power.is_null() {
        return SocketStatus::NullPointer;
    }
    with_socket(
        unsafe { socket.as_ref() },
        SmartSocket::current_power,
        |value| unsafe { power.write(value) },
    )
}

fn with_socket<T>(
    socket: Option<&SmartSocket>,
    request: impl FnOnce(&SmartSocket) -> Result<T, DeviceError>,
    store: impl FnOnce(T),
) -> SocketStatus {
    let Some(socket) = socket else {
        return SocketStatus::NullPointer;
    };
    match request(socket) {
        Ok(value) => {
            store(value);
            SocketStatus::Ok
        }
        Err(e) => SocketStatus::from(&e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    fn state(socket: *const SmartSocket) -> (bool, f64) {
        let mut is_on = false;
        let mut power = -1.0;
        unsafe {
            assert_eq!(smart_socket_is_on(socket, &raw mut is_on), SocketStatus::Ok);
            assert_eq!(smart_socket_power(socket, &raw mut power), SocketStatus::Ok);
        }
        (is_on, power)
    }

    #[test]
    fn mock_socket_turns_on_and_off() {
        let socket = smart_socket_new_mock(1500.0);
        assert_eq!(state(socket), (false, 0.0));

        assert_eq!(unsafe { smart_socket_turn_on(socket) }, SocketStatus::Ok);
        assert_eq!(state(socket), (true, 1500.0));

        assert_eq!(unsafe { smart_socket_turn_off(socket) }, SocketStatus::Ok);
        assert_eq!(state(socket), (false, 0.0));

        unsafe { smart_socket_free(socket) };
    }

    #[test]
    fn null_pointers_are_reported_instead_of_crashing() {
        let mut value = 0.0;
        unsafe {
            assert_eq!(smart_socket_turn_on(ptr::null()), SocketStatus::NullPointer);
            assert_eq!(
                smart_socket_power(ptr::null(), &raw mut value),
                SocketStatus::NullPointer
            );
            smart_socket_free(ptr::null_mut());
            assert!(smart_socket_new_tcp(ptr::null()).is_null());
        }

        let socket = smart_socket_new_mock(10.0);
        unsafe {
            assert_eq!(
                smart_socket_is_on(socket, ptr::null_mut()),
                SocketStatus::NullPointer
            );
            smart_socket_free(socket);
        }
    }

    #[test]
    fn unreachable_device_is_a_connection_error() {
        let socket = unsafe { smart_socket_new_tcp(c"127.0.0.1:1".as_ptr()) };
        assert!(!socket.is_null());
        assert_eq!(
            unsafe { smart_socket_turn_on(socket) },
            SocketStatus::ConnectionError
        );
        unsafe { smart_socket_free(socket) };
    }
}

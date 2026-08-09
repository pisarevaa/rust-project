/// Умная розетка, управляемая синхронно по TCP.
use crate::error::DeviceError;
use crate::protocol::{Command, Response, RESPONSE_LEN};
use std::fmt;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Mutex;
use std::time::Duration;

/// Сколько ждать ответа розетки, прежде чем считать ее недоступной.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

/// Канал связи с розеткой.
pub trait SocketTransport: fmt::Debug + Send + Sync {
    fn request(&self, command: Command) -> Result<Response, DeviceError>;
}

/// Реальный обмен с розеткой по TCP.
#[derive(Debug)]
pub struct TcpTransport {
    stream: Mutex<TcpStream>,
}

impl TcpTransport {
    pub fn connect(address: impl ToSocketAddrs) -> Result<Self, DeviceError> {
        let stream = TcpStream::connect(address)?;
        stream.set_read_timeout(Some(REQUEST_TIMEOUT))?;
        stream.set_write_timeout(Some(REQUEST_TIMEOUT))?;
        Ok(Self {
            stream: Mutex::new(stream),
        })
    }
}

impl SocketTransport for TcpTransport {
    fn request(&self, command: Command) -> Result<Response, DeviceError> {
        let mut stream = self
            .stream
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        stream.write_all(&[command.to_byte()]).map_err(as_timeout)?;
        stream.flush().map_err(as_timeout)?;

        let mut buffer = [0_u8; RESPONSE_LEN];
        stream.read_exact(&mut buffer).map_err(as_timeout)?;
        Response::from_bytes(buffer)
    }
}

fn as_timeout(e: std::io::Error) -> DeviceError {
    if matches!(e.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) {
        DeviceError::Timeout
    } else {
        DeviceError::Io(e)
    }
}

/// Обмен по TCP с переподключением на каждый запрос.
#[derive(Debug)]
pub struct ReconnectingTcpTransport {
    address: String,
}

impl ReconnectingTcpTransport {
    #[must_use]
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
        }
    }
}

impl SocketTransport for ReconnectingTcpTransport {
    fn request(&self, command: Command) -> Result<Response, DeviceError> {
        TcpTransport::connect(self.address.as_str())?.request(command)
    }
}

#[derive(Debug)]
pub struct MockTransport {
    state: Mutex<MockState>,
}

#[derive(Debug)]
struct MockState {
    is_on: bool,
    power: f64,
}

impl MockTransport {
    #[must_use]
    pub fn new(power: f64) -> Self {
        Self {
            state: Mutex::new(MockState {
                is_on: false,
                power,
            }),
        }
    }
}

impl SocketTransport for MockTransport {
    fn request(&self, command: Command) -> Result<Response, DeviceError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match command {
            Command::TurnOn => state.is_on = true,
            Command::TurnOff => state.is_on = false,
            Command::Status => {}
        }
        Ok(Response {
            is_on: state.is_on,
            power: if state.is_on { state.power } else { 0.0 },
        })
    }
}

#[derive(Debug)]
pub struct SmartSocket {
    transport: Box<dyn SocketTransport>,
}

impl SmartSocket {
    #[must_use]
    pub fn new(transport: impl SocketTransport + 'static) -> Self {
        Self {
            transport: Box::new(transport),
        }
    }

    pub fn connect(address: impl ToSocketAddrs) -> Result<Self, DeviceError> {
        Ok(Self::new(TcpTransport::connect(address)?))
    }

    #[must_use]
    pub fn reconnecting(address: impl Into<String>) -> Self {
        Self::new(ReconnectingTcpTransport::new(address))
    }

    #[must_use]
    pub fn mock(power: f64) -> Self {
        Self::new(MockTransport::new(power))
    }

    pub fn status(&self) -> Result<Response, DeviceError> {
        self.transport.request(Command::Status)
    }

    pub fn turn_on(&self) -> Result<(), DeviceError> {
        self.transport.request(Command::TurnOn).map(|_| ())
    }

    pub fn turn_off(&self) -> Result<(), DeviceError> {
        self.transport.request(Command::TurnOff).map(|_| ())
    }

    pub fn is_on(&self) -> Result<bool, DeviceError> {
        Ok(self.status()?.is_on)
    }

    pub fn current_power(&self) -> Result<f64, DeviceError> {
        Ok(self.status()?.power)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_socket_is_off_with_zero_power() {
        let s = SmartSocket::mock(100.0);
        assert!(!s.is_on().expect("имитация всегда отвечает"));
        assert!((s.current_power().expect("имитация всегда отвечает")).abs() < f64::EPSILON);
    }

    #[test]
    fn turn_on_reports_power_and_state() {
        let s = SmartSocket::mock(100.0);
        s.turn_on().expect("имитация всегда отвечает");
        assert!(s.is_on().expect("имитация всегда отвечает"));
        assert!(
            (s.current_power().expect("имитация всегда отвечает") - 100.0).abs() < f64::EPSILON
        );
    }

    #[test]
    fn turn_off_resets_power() {
        let s = SmartSocket::mock(100.0);
        s.turn_on().expect("имитация всегда отвечает");
        s.turn_off().expect("имитация всегда отвечает");
        assert!(!s.is_on().expect("имитация всегда отвечает"));
        assert!((s.current_power().expect("имитация всегда отвечает")).abs() < f64::EPSILON);
    }

    #[test]
    fn status_does_not_change_state() {
        let s = SmartSocket::mock(100.0);
        s.turn_on().expect("имитация всегда отвечает");
        let first = s.status().expect("имитация всегда отвечает");
        let second = s.status().expect("имитация всегда отвечает");
        assert_eq!(first, second);
        assert!(first.is_on);
    }

    #[test]
    fn connecting_to_a_closed_port_fails() {
        let error = SmartSocket::connect("127.0.0.1:1").expect_err("соединение невозможно");
        assert!(matches!(error, DeviceError::Io(_)));
    }

    #[test]
    fn a_silent_peer_ends_in_a_timeout_instead_of_hanging() {
        use std::net::TcpListener;

        let listener = TcpListener::bind("127.0.0.1:0").expect("адрес свободен");
        let address = listener.local_addr().expect("адрес известен");
        let _accepted = std::thread::spawn(move || listener.accept());

        let transport = TcpTransport::connect(address).expect("соединение принято");

        transport
            .stream
            .lock()
            .expect("мьютекс свободен")
            .set_read_timeout(Some(Duration::from_millis(200)))
            .expect("таймаут выставлен");

        let error = transport
            .request(Command::Status)
            .expect_err("ответа не будет");
        assert!(matches!(error, DeviceError::Timeout));
        assert_eq!(error.to_string(), "устройство не ответило вовремя");
    }

    #[test]
    fn a_reconnecting_socket_is_created_even_without_a_device() {
        let socket = SmartSocket::reconnecting("127.0.0.1:1");
        assert!(matches!(socket.status(), Err(DeviceError::Io(_))));
    }

    #[test]
    fn a_failing_transport_surfaces_its_error() {
        #[derive(Debug)]
        struct Unavailable;

        impl SocketTransport for Unavailable {
            fn request(&self, _: Command) -> Result<Response, DeviceError> {
                Err(DeviceError::NoData)
            }
        }

        let s = SmartSocket::new(Unavailable);
        assert!(matches!(s.current_power(), Err(DeviceError::NoData)));
    }
}

//! Умный термометр, получающий температуру пакетами UDP.

use crate::error::DeviceError;
use crate::protocol::{decode_temperature, TEMPERATURE_PACKET_LEN};
use std::fmt;
use std::io::ErrorKind;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const POLL_TIMEOUT: Duration = Duration::from_millis(100);

pub trait TemperatureSource: fmt::Debug + Send + Sync {
    /// Возвращает последнее известное показание температуры.
    ///
    /// # Errors
    ///
    /// Возвращает [`DeviceError::NoData`], если ни одного показания
    /// еще не получено.
    fn temperature(&self) -> Result<f64, DeviceError>;
}

#[derive(Debug)]
pub struct UdpTemperatureSource {
    address: SocketAddr,
    last: Arc<Mutex<Option<f64>>>,
    running: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl UdpTemperatureSource {
    /// Занимает UDP-адрес и запускает фоновый прием показаний.
    ///
    /// # Errors
    ///
    /// Возвращает [`DeviceError::Io`], если адрес занят, не удалось выставить
    /// таймаут чтения или узнать локальный адрес сокета.
    pub fn bind(address: impl ToSocketAddrs) -> Result<Self, DeviceError> {
        let socket = UdpSocket::bind(address)?;
        // Таймаут нужен, чтобы поток регулярно просыпался и проверял флаг остановки.
        socket.set_read_timeout(Some(POLL_TIMEOUT))?;
        let address = socket.local_addr()?;

        let last = Arc::new(Mutex::new(None));
        let running = Arc::new(AtomicBool::new(true));

        let worker = {
            let last = Arc::clone(&last);
            let running = Arc::clone(&running);
            thread::spawn(move || receive_loop(&socket, &last, &running))
        };

        Ok(Self {
            address,
            last,
            running,
            worker: Some(worker),
        })
    }

    #[must_use]
    pub fn local_addr(&self) -> SocketAddr {
        self.address
    }
}

impl TemperatureSource for UdpTemperatureSource {
    fn temperature(&self) -> Result<f64, DeviceError> {
        let last = self
            .last
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        last.ok_or(DeviceError::NoData)
    }
}

impl Drop for UdpTemperatureSource {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn receive_loop(socket: &UdpSocket, last: &Mutex<Option<f64>>, running: &AtomicBool) {
    let mut buffer = [0_u8; TEMPERATURE_PACKET_LEN];
    while running.load(Ordering::Relaxed) {
        match socket.recv_from(&mut buffer) {
            Ok((length, _)) => {
                if let Ok(temperature) = decode_temperature(&buffer[..length]) {
                    *last
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(temperature);
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut => {}
            Err(_) => break,
        }
    }
}

#[derive(Debug, Default)]
pub struct MockTemperatureSource {
    temperature: Mutex<Option<f64>>,
}

impl MockTemperatureSource {
    #[must_use]
    pub fn new(temperature: f64) -> Self {
        Self {
            temperature: Mutex::new(Some(temperature)),
        }
    }

    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn set(&self, temperature: f64) {
        *self
            .temperature
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(temperature);
    }
}

impl TemperatureSource for MockTemperatureSource {
    fn temperature(&self) -> Result<f64, DeviceError> {
        let temperature = self
            .temperature
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        temperature.ok_or(DeviceError::NoData)
    }
}

#[derive(Debug)]
pub struct SmartThermometer {
    source: Box<dyn TemperatureSource>,
}

impl SmartThermometer {
    #[must_use]
    pub fn new(source: impl TemperatureSource + 'static) -> Self {
        Self {
            source: Box::new(source),
        }
    }

    /// Создает термометр, слушающий показания по UDP.
    ///
    /// # Errors
    ///
    /// Возвращает [`DeviceError::Io`], если занять адрес не удалось.
    pub fn bind(address: impl ToSocketAddrs) -> Result<Self, DeviceError> {
        Ok(Self::new(UdpTemperatureSource::bind(address)?))
    }

    #[must_use]
    pub fn mock(temperature: f64) -> Self {
        Self::new(MockTemperatureSource::new(temperature))
    }

    /// Возвращает последнее полученное показание.
    ///
    /// # Errors
    ///
    /// Возвращает [`DeviceError::NoData`], если показаний еще не было.
    pub fn temperature(&self) -> Result<f64, DeviceError> {
        self.source.temperature()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::encode_temperature;
    use std::time::Instant;

    fn wait_for_reading(thermometer: &SmartThermometer) -> Result<f64, DeviceError> {
        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            match thermometer.temperature() {
                Ok(value) => return Ok(value),
                Err(e) if Instant::now() >= deadline => return Err(e),
                Err(_) => thread::sleep(Duration::from_millis(10)),
            }
        }
    }

    #[test]
    fn returns_mocked_temperature() {
        let t = SmartThermometer::mock(21.5);
        assert!((t.temperature().expect("значение задано") - 21.5).abs() < f64::EPSILON);
    }

    #[test]
    fn reports_missing_data_before_the_first_reading() {
        let t = SmartThermometer::new(MockTemperatureSource::empty());
        assert!(matches!(t.temperature(), Err(DeviceError::NoData)));
    }

    #[test]
    fn mocked_source_returns_the_latest_value() {
        let source = MockTemperatureSource::new(20.0);
        source.set(25.0);
        assert!((source.temperature().expect("значение задано") - 25.0).abs() < f64::EPSILON);
    }

    #[test]
    fn udp_source_reports_missing_data_before_the_first_packet() {
        let t = SmartThermometer::bind("127.0.0.1:0").expect("адрес свободен");
        assert!(matches!(t.temperature(), Err(DeviceError::NoData)));
    }

    #[test]
    fn udp_source_receives_a_packet() {
        let source = UdpTemperatureSource::bind("127.0.0.1:0").expect("адрес свободен");
        let address = source.local_addr();
        let thermometer = SmartThermometer::new(source);

        let sender = UdpSocket::bind("127.0.0.1:0").expect("адрес свободен");
        sender
            .send_to(&encode_temperature(23.5), address)
            .expect("пакет отправлен");

        let received = wait_for_reading(&thermometer).expect("пакет должен дойти");
        assert!((received - 23.5).abs() < f64::EPSILON);
    }

    #[test]
    fn udp_source_keeps_the_latest_value() {
        let source = UdpTemperatureSource::bind("127.0.0.1:0").expect("адрес свободен");
        let address = source.local_addr();
        let thermometer = SmartThermometer::new(source);

        let sender = UdpSocket::bind("127.0.0.1:0").expect("адрес свободен");
        sender
            .send_to(&encode_temperature(10.0), address)
            .expect("пакет отправлен");
        wait_for_reading(&thermometer).expect("первый пакет должен дойти");

        sender
            .send_to(&encode_temperature(30.0), address)
            .expect("пакет отправлен");

        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            let value = thermometer.temperature().expect("показание уже есть");
            if (value - 30.0).abs() < f64::EPSILON {
                break;
            }
            assert!(Instant::now() < deadline, "второй пакет так и не дошел");
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn packets_of_a_wrong_size_are_ignored() {
        let source = UdpTemperatureSource::bind("127.0.0.1:0").expect("адрес свободен");
        let address = source.local_addr();
        let thermometer = SmartThermometer::new(source);

        let sender = UdpSocket::bind("127.0.0.1:0").expect("адрес свободен");
        sender.send_to(b"abc", address).expect("пакет отправлен");
        thread::sleep(Duration::from_millis(100));

        assert!(matches!(
            thermometer.temperature(),
            Err(DeviceError::NoData)
        ));
    }

    #[test]
    fn dropping_the_source_stops_the_thread_and_frees_the_port() {
        let source = UdpTemperatureSource::bind("127.0.0.1:0").expect("адрес свободен");
        let address = source.local_addr();
        drop(source);

        UdpSocket::bind(address).expect("порт освобожден вместе с потоком");
    }
}

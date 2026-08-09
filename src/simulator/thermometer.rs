/// Имитатор умного термометра: неблокирующая отправка UDP-пакетов.

use crate::config::SimulatorConfig;
use crate::error::DeviceError;
use crate::protocol::encode_temperature;
use std::io::ErrorKind;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

const MIN_TEMPERATURE: f64 = 15.0;
const MAX_TEMPERATURE: f64 = 30.0;

#[derive(Debug)]
pub struct ThermometerSimulator {
    socket: UdpSocket,
    target: SocketAddr,
    config: SimulatorConfig,
    generator: TemperatureGenerator,
}

impl ThermometerSimulator {
    pub fn new(config: SimulatorConfig) -> Result<Self, DeviceError> {
        let target = config
            .address
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| DeviceError::Protocol(format!("адрес «{}» пуст", config.address)))?;

        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_nonblocking(true)?;

        Ok(Self {
            socket,
            target,
            config,
            generator: TemperatureGenerator::from_clock(),
        })
    }

    #[must_use]
    pub fn target(&self) -> SocketAddr {
        self.target
    }

    pub fn send_next(&mut self) -> Result<f64, DeviceError> {
        let temperature = self.generator.next_temperature();
        match self
            .socket
            .send_to(&encode_temperature(temperature), self.target)
        {
            Ok(_) => Ok(temperature),
            Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(temperature),
            Err(e) => Err(e.into()),
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            match self.send_next() {
                Ok(temperature) => {
                    println!("{:.1} °C → {}", temperature, self.target);
                }
                Err(e) => eprintln!("не удалось отправить пакет: {e}"),
            }
            thread::sleep(self.config.period);
        }
    }
}

#[derive(Debug)]
pub struct TemperatureGenerator {
    state: u64,
}

impl TemperatureGenerator {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    #[must_use]
    pub fn from_clock() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(1, |d| u64::try_from(d.as_nanos()).unwrap_or(1));
        Self::new(seed)
    }

    #[allow(clippy::cast_precision_loss)]
    pub fn next_temperature(&mut self) -> f64 {
        let fraction = (self.next_u64() >> 11) as f64 / (1_u64 << 53) as f64;
        MIN_TEMPERATURE + fraction * (MAX_TEMPERATURE - MIN_TEMPERATURE)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::TEMPERATURE_PACKET_LEN;
    use crate::thermometer::{SmartThermometer, UdpTemperatureSource};
    use std::time::{Duration, Instant};

    fn config_for(address: SocketAddr) -> SimulatorConfig {
        SimulatorConfig {
            address: address.to_string(),
            period: Duration::from_millis(10),
        }
    }

    #[test]
    fn generated_temperature_stays_within_bounds() {
        let mut generator = TemperatureGenerator::new(42);
        for _ in 0..1000 {
            let t = generator.next_temperature();
            assert!(
                (MIN_TEMPERATURE..=MAX_TEMPERATURE).contains(&t),
                "{t} вне допустимого диапазона"
            );
        }
    }

    #[test]
    fn generator_does_not_repeat_itself_immediately() {
        let mut generator = TemperatureGenerator::new(42);
        let first = generator.next_temperature();
        let second = generator.next_temperature();
        assert!((first - second).abs() > f64::EPSILON);
    }

    #[test]
    fn a_zero_seed_still_produces_varying_values() {
        let mut generator = TemperatureGenerator::new(0);
        let first = generator.next_temperature();
        let second = generator.next_temperature();
        assert!((first - second).abs() > f64::EPSILON);
    }

    #[test]
    fn the_same_seed_gives_the_same_sequence() {
        let mut left = TemperatureGenerator::new(7);
        let mut right = TemperatureGenerator::new(7);
        assert!((left.next_temperature() - right.next_temperature()).abs() < f64::EPSILON);
    }

    #[test]
    fn resolves_the_target_address_from_the_config() {
        let receiver = UdpSocket::bind("127.0.0.1:0").expect("адрес свободен");
        let address = receiver.local_addr().expect("адрес известен");

        let simulator = ThermometerSimulator::new(config_for(address)).expect("сокет создан");
        assert_eq!(simulator.target(), address);
    }

    #[test]
    fn reports_an_unresolvable_address() {
        let config = SimulatorConfig {
            address: "без-порта".to_string(),
            period: Duration::from_millis(10),
        };
        assert!(ThermometerSimulator::new(config).is_err());
    }

    #[test]
    fn sends_a_packet_of_the_expected_size() {
        let listener = UdpSocket::bind("127.0.0.1:0").expect("адрес свободен");
        listener
            .set_read_timeout(Some(Duration::from_secs(1)))
            .expect("таймаут выставлен");
        let address = listener.local_addr().expect("адрес известен");

        let mut simulator = ThermometerSimulator::new(config_for(address)).expect("сокет создан");
        let sent = simulator.send_next().expect("пакет отправлен");

        let mut buffer = [0_u8; 64];
        let (length, _) = listener.recv_from(&mut buffer).expect("пакет дошел");
        assert_eq!(length, TEMPERATURE_PACKET_LEN);

        let received = crate::protocol::decode_temperature(&buffer[..length]).expect("пакет цел");
        assert!((received - sent).abs() < f64::EPSILON);
    }

    #[test]
    fn a_thermometer_receives_what_the_simulator_sends() {
        let source = UdpTemperatureSource::bind("127.0.0.1:0").expect("адрес свободен");
        let address = source.local_addr();
        let thermometer = SmartThermometer::new(source);

        let mut simulator = ThermometerSimulator::new(config_for(address)).expect("сокет создан");
        let sent = simulator.send_next().expect("пакет отправлен");

        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            if let Ok(received) = thermometer.temperature() {
                assert!((received - sent).abs() < f64::EPSILON);
                break;
            }
            assert!(Instant::now() < deadline, "пакет так и не дошел");
            thread::sleep(Duration::from_millis(10));
        }
    }
}

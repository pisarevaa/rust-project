use crate::report::Report;
use crate::socket::SmartSocket;
use crate::thermometer::SmartThermometer;

#[derive(Debug)]
pub enum SmartDevice {
    Thermometer(SmartThermometer),
    Socket(SmartSocket),
}

impl From<SmartThermometer> for SmartDevice {
    fn from(thermometer: SmartThermometer) -> Self {
        Self::Thermometer(thermometer)
    }
}

impl From<SmartSocket> for SmartDevice {
    fn from(socket: SmartSocket) -> Self {
        Self::Socket(socket)
    }
}

impl Report for SmartDevice {
    fn report(&self) -> String {
        match self {
            Self::Thermometer(t) => t.report(),
            Self::Socket(s) => s.report(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DeviceError;
    use crate::protocol::{Command, Response};
    use crate::socket::SocketTransport;
    use crate::thermometer::{MockTemperatureSource, TemperatureSource};

    /// Устройство, до которого не удается достучаться.
    #[derive(Debug)]
    struct Unavailable;

    impl SocketTransport for Unavailable {
        fn request(&self, _: Command) -> Result<Response, DeviceError> {
            Err(DeviceError::Protocol("пустой ответ".to_string()))
        }
    }

    impl TemperatureSource for Unavailable {
        fn temperature(&self) -> Result<f64, DeviceError> {
            Err(DeviceError::NoData)
        }
    }

    #[test]
    fn thermometer_converts_into_device() {
        let device: SmartDevice = SmartThermometer::mock(20.0).into();
        match device {
            SmartDevice::Thermometer(t) => {
                assert!((t.temperature().expect("имитация отвечает") - 20.0).abs() < f64::EPSILON);
            }
            SmartDevice::Socket(_) => panic!("ожидался термометр"),
        }
    }

    #[test]
    fn socket_converts_into_device() {
        let device: SmartDevice = SmartSocket::mock(50.0).into();
        match device {
            SmartDevice::Socket(s) => {
                assert!(!s.is_on().expect("имитация отвечает"));
            }
            SmartDevice::Thermometer(_) => panic!("ожидалась розетка"),
        }
    }

    #[test]
    fn thermometer_report_shows_temperature() {
        let device: SmartDevice = SmartThermometer::mock(21.5).into();
        assert_eq!(device.report(), "термометр показывает 21.5 °C");
    }

    #[test]
    fn switched_on_socket_report_shows_power() {
        let socket = SmartSocket::mock(150.0);
        socket.turn_on().expect("имитация отвечает");
        let device: SmartDevice = socket.into();
        assert_eq!(device.report(), "розетка включена, мощность 150.0 Вт");
    }

    #[test]
    fn switched_off_socket_report_shows_zero_power() {
        let device: SmartDevice = SmartSocket::mock(150.0).into();
        assert_eq!(device.report(), "розетка выключена, мощность 0.0 Вт");
    }

    #[test]
    fn unreachable_socket_reports_the_error() {
        let device: SmartDevice = SmartSocket::new(Unavailable).into();
        assert_eq!(
            device.report(),
            "розетка недоступна: ошибка протокола: пустой ответ"
        );
    }

    #[test]
    fn thermometer_without_readings_reports_the_error() {
        let device: SmartDevice = SmartThermometer::new(Unavailable).into();
        assert_eq!(
            device.report(),
            "термометр недоступен: данные еще не получены"
        );
    }

    #[test]
    fn thermometer_report_rounds_to_one_decimal() {
        let device: SmartDevice = SmartThermometer::new(MockTemperatureSource::new(20.04)).into();
        assert_eq!(device.report(), "термометр показывает 20.0 °C");
    }
}

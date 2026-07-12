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
            Self::Thermometer(t) => {
                format!("термометр показывает {:.1} °C", t.temperature())
            }
            Self::Socket(s) => {
                let state = if s.is_on() {
                    "включена"
                } else {
                    "выключена"
                };
                format!("розетка {state}, мощность {:.1} Вт", s.current_power())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thermometer_converts_into_device() {
        let device: SmartDevice = SmartThermometer::new(20.0).into();
        match device {
            SmartDevice::Thermometer(t) => {
                assert!((t.temperature() - 20.0).abs() < f64::EPSILON);
            }
            SmartDevice::Socket(_) => panic!("ожидался термометр"),
        }
    }

    #[test]
    fn socket_converts_into_device() {
        let device: SmartDevice = SmartSocket::new(50.0).into();
        match device {
            SmartDevice::Socket(s) => {
                assert!(!s.is_on());
            }
            SmartDevice::Thermometer(_) => panic!("ожидалась розетка"),
        }
    }

    #[test]
    fn thermometer_report_shows_temperature() {
        let device: SmartDevice = SmartThermometer::new(21.5).into();
        assert_eq!(device.report(), "термометр показывает 21.5 °C");
    }

    #[test]
    fn switched_on_socket_report_shows_power() {
        let mut socket = SmartSocket::new(150.0);
        socket.turn_on();
        let device: SmartDevice = socket.into();
        assert_eq!(device.report(), "розетка включена, мощность 150.0 Вт");
    }

    #[test]
    fn switched_off_socket_report_shows_zero_power() {
        let device: SmartDevice = SmartSocket::new(150.0).into();
        assert_eq!(device.report(), "розетка выключена, мощность 0.0 Вт");
    }
}

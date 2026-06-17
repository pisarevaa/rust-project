use crate::socket::SmartSocket;
use crate::thermometer::SmartThermometer;

pub enum SmartDevice {
    Thermometer(SmartThermometer),
    Socket(SmartSocket),
}

impl SmartDevice {
    pub fn report(&self) {
        match self {
            SmartDevice::Thermometer(t) => {
                println!("  Термометр «{}»: {:.1}°C", t.name(), t.temperature());
            }
            SmartDevice::Socket(s) => {
                let state = if s.is_on() {
                    "включена"
                } else {
                    "выключена"
                };
                println!(
                    "  Розетка «{}»: {}, мощность {:.1} Вт",
                    s.name(),
                    state,
                    s.current_power()
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::SmartSocket;
    use crate::thermometer::SmartThermometer;

    #[test]
    fn thermometer_variant_exposes_temperature() {
        let d = SmartDevice::Thermometer(SmartThermometer::new("Т".to_string(), 20.0));
        d.report();
        match d {
            SmartDevice::Thermometer(t) => assert_eq!(t.temperature(), 20.0),
            SmartDevice::Socket(_) => panic!("ожидался термометр"),
        }
    }

    #[test]
    fn socket_variant_reflects_state() {
        let mut socket = SmartSocket::new("Р".to_string(), 50.0);
        socket.turn_on();
        let d = SmartDevice::Socket(socket);
        d.report();
        match d {
            SmartDevice::Socket(s) => {
                assert!(s.is_on());
                assert_eq!(s.current_power(), 50.0);
            }
            SmartDevice::Thermometer(_) => panic!("ожидалась розетка"),
        }
    }
}

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
    fn wraps_thermometer() {
        let d = SmartDevice::Thermometer(SmartThermometer::new("Т".to_string(), 20.0));
        d.report();
        assert!(matches!(d, SmartDevice::Thermometer(_)));
    }

    #[test]
    fn wraps_socket() {
        let d = SmartDevice::Socket(SmartSocket::new("Р".to_string(), 50.0));
        d.report();
        assert!(matches!(d, SmartDevice::Socket(_)));
    }
}

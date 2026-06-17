use crate::device::SmartDevice;

pub struct Room {
    name: String,
    devices: Vec<SmartDevice>,
}

impl Room {
    pub fn new(name: String, devices: Vec<SmartDevice>) -> Self {
        Self { name, devices }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn device(&self, index: usize) -> &SmartDevice {
        &self.devices[index]
    }

    pub fn device_mut(&mut self, index: usize) -> &mut SmartDevice {
        &mut self.devices[index]
    }

    pub fn report(&self) {
        println!("Комната «{}»:", self.name);
        for device in &self.devices {
            device.report();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::SmartSocket;
    use crate::thermometer::SmartThermometer;

    fn sample_room() -> Room {
        Room::new(
            "Кухня".to_string(),
            vec![
                SmartDevice::Thermometer(SmartThermometer::new("Т".to_string(), 22.0)),
                SmartDevice::Socket(SmartSocket::new("Р".to_string(), 60.0)),
            ],
        )
    }

    #[test]
    fn device_returns_reference_by_index() {
        let room = sample_room();
        assert!(matches!(room.device(0), SmartDevice::Thermometer(_)));
        assert!(matches!(room.device(1), SmartDevice::Socket(_)));
    }

    #[test]
    fn device_mut_allows_mutation() {
        let mut room = sample_room();
        if let SmartDevice::Socket(s) = room.device_mut(1) {
            s.turn_on();
            assert!(s.is_on());
        } else {
            panic!("ожидалась розетка по индексу 1");
        }
    }

    #[test]
    #[should_panic]
    fn device_out_of_bounds_panics() {
        let room = sample_room();
        room.device(99);
    }
}

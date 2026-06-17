use crate::room::Room;

pub struct SmartHouse {
    name: String,
    rooms: Vec<Room>,
}

impl SmartHouse {
    pub fn new(name: String, rooms: Vec<Room>) -> Self {
        Self { name, rooms }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn room(&self, index: usize) -> &Room {
        &self.rooms[index]
    }

    pub fn room_mut(&mut self, index: usize) -> &mut Room {
        &mut self.rooms[index]
    }

    pub fn report(&self) {
        println!("=== Отчет о доме «{}» ===", self.name);
        for room in &self.rooms {
            room.report();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::SmartDevice;
    use crate::room::Room;
    use crate::socket::SmartSocket;

    fn sample_house() -> SmartHouse {
        SmartHouse::new(
            "Дом".to_string(),
            vec![Room::new(
                "Кухня".to_string(),
                vec![SmartDevice::Socket(SmartSocket::new("Р".to_string(), 60.0))],
            )],
        )
    }

    #[test]
    fn room_returns_reference_by_index() {
        let house = sample_house();
        assert_eq!(house.room(0).name(), "Кухня");
    }

    #[test]
    fn room_mut_allows_mutation() {
        let mut house = sample_house();
        if let SmartDevice::Socket(s) = house.room_mut(0).device_mut(0) {
            s.turn_on();
            assert!(s.is_on());
        } else {
            panic!("ожидалась розетка");
        }
    }

    #[test]
    #[should_panic]
    fn room_out_of_bounds_panics() {
        let house = sample_house();
        house.room(99);
    }
}

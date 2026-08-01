use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum SmartHouseError {
    RoomNotFound { room: String },
    DeviceNotFound { room: String, device: String },
}

impl fmt::Display for SmartHouseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RoomNotFound { room } => write!(f, "комната «{room}» не найдена"),
            Self::DeviceNotFound { room, device } => {
                write!(f, "устройство «{device}» не найдено в комнате «{room}»")
            }
        }
    }
}

impl std::error::Error for SmartHouseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn room_not_found_names_the_room() {
        let e = SmartHouseError::RoomNotFound {
            room: "Кухня".to_string(),
        };
        assert_eq!(e.to_string(), "комната «Кухня» не найдена");
    }

    #[test]
    fn device_not_found_names_room_and_device() {
        let e = SmartHouseError::DeviceNotFound {
            room: "Кухня".to_string(),
            device: "Розетка".to_string(),
        };
        assert_eq!(
            e.to_string(),
            "устройство «Розетка» не найдено в комнате «Кухня»"
        );
    }

    #[test]
    fn converts_into_boxed_std_error() {
        let e = SmartHouseError::RoomNotFound {
            room: "Кухня".to_string(),
        };
        let boxed: Box<dyn std::error::Error> = Box::new(e);
        assert_eq!(boxed.to_string(), "комната «Кухня» не найдена");
    }
}

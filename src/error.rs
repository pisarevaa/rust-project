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

#[derive(Debug)]
pub enum DeviceError {
    Io(std::io::Error),
    Protocol(String),
    Timeout,
    NoData,
}

impl fmt::Display for DeviceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "ошибка соединения: {e}"),
            Self::Protocol(details) => write!(f, "ошибка протокола: {details}"),
            Self::Timeout => write!(f, "устройство не ответило вовремя"),
            Self::NoData => write!(f, "данные еще не получены"),
        }
    }
}

impl std::error::Error for DeviceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Protocol(_) | Self::Timeout | Self::NoData => None,
        }
    }
}

impl From<std::io::Error> for DeviceError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use std::io::ErrorKind;

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
        let boxed: Box<dyn Error> = Box::new(e);
        assert_eq!(boxed.to_string(), "комната «Кухня» не найдена");
    }

    #[test]
    fn io_error_converts_into_device_error() {
        let e: DeviceError = std::io::Error::new(ErrorKind::ConnectionRefused, "отказано").into();
        assert_eq!(e.to_string(), "ошибка соединения: отказано");
    }

    #[test]
    fn io_error_is_exposed_as_a_source() {
        let e: DeviceError = std::io::Error::new(ErrorKind::ConnectionRefused, "отказано").into();
        assert!(e.source().is_some());
    }

    #[test]
    fn missing_data_has_no_source() {
        assert!(DeviceError::NoData.source().is_none());
        assert_eq!(DeviceError::NoData.to_string(), "данные еще не получены");
    }

    #[test]
    fn device_error_converts_into_boxed_std_error() {
        let boxed: Box<dyn Error> = Box::new(DeviceError::Protocol("пустой ответ".to_string()));
        assert_eq!(boxed.to_string(), "ошибка протокола: пустой ответ");
    }
}

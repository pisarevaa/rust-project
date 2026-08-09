//! Протокол обмена с умной розеткой поверх TCP.

use crate::error::DeviceError;

pub const REQUEST_LEN: usize = 1;
pub const RESPONSE_LEN: usize = 9;
pub const TEMPERATURE_PACKET_LEN: usize = 8;

#[must_use]
pub fn encode_temperature(temperature: f64) -> [u8; TEMPERATURE_PACKET_LEN] {
    temperature.to_be_bytes()
}

/// Восстанавливает температуру из UDP-пакета.
///
/// # Errors
///
/// Возвращает [`DeviceError::Protocol`], если длина пакета
/// отличается от [`TEMPERATURE_PACKET_LEN`].
pub fn decode_temperature(packet: &[u8]) -> Result<f64, DeviceError> {
    let bytes: [u8; TEMPERATURE_PACKET_LEN] = packet.try_into().map_err(|_| {
        DeviceError::Protocol(format!(
            "ожидался пакет из {TEMPERATURE_PACKET_LEN} байт, получено {}",
            packet.len()
        ))
    })?;
    Ok(f64::from_be_bytes(bytes))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    TurnOn,
    TurnOff,
    Status,
}

impl Command {
    #[must_use]
    pub fn to_byte(self) -> u8 {
        match self {
            Self::TurnOn => 0,
            Self::TurnOff => 1,
            Self::Status => 2,
        }
    }

    /// Разбирает код команды.
    ///
    /// # Errors
    ///
    /// Возвращает [`DeviceError::Protocol`], если байт не соответствует
    /// ни одной известной команде.
    pub fn from_byte(byte: u8) -> Result<Self, DeviceError> {
        match byte {
            0 => Ok(Self::TurnOn),
            1 => Ok(Self::TurnOff),
            2 => Ok(Self::Status),
            other => Err(DeviceError::Protocol(format!(
                "неизвестный код команды: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Response {
    pub is_on: bool,
    pub power: f64,
}

impl Response {
    #[must_use]
    pub fn to_bytes(self) -> [u8; RESPONSE_LEN] {
        let mut bytes = [0_u8; RESPONSE_LEN];
        bytes[0] = u8::from(self.is_on);
        bytes[1..].copy_from_slice(&self.power.to_be_bytes());
        bytes
    }

    /// Разбирает ответ розетки.
    ///
    /// # Errors
    ///
    /// Возвращает [`DeviceError::Protocol`], если байт состояния
    /// отличается от 0 и 1.
    pub fn from_bytes(bytes: [u8; RESPONSE_LEN]) -> Result<Self, DeviceError> {
        let is_on = match bytes[0] {
            0 => false,
            1 => true,
            other => {
                return Err(DeviceError::Protocol(format!(
                    "недопустимый байт состояния: {other}"
                )))
            }
        };
        let mut power = [0_u8; 8];
        power.copy_from_slice(&bytes[1..]);
        Ok(Self {
            is_on,
            power: f64::from_be_bytes(power),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_survives_a_round_trip() {
        for command in [Command::TurnOn, Command::TurnOff, Command::Status] {
            let byte = command.to_byte();
            assert_eq!(Command::from_byte(byte).expect("код известен"), command);
        }
    }

    #[test]
    fn unknown_command_byte_is_rejected() {
        let error = Command::from_byte(200).expect_err("код 200 не определен");
        assert_eq!(
            error.to_string(),
            "ошибка протокола: неизвестный код команды: 200"
        );
    }

    #[test]
    fn response_survives_a_round_trip() {
        let response = Response {
            is_on: true,
            power: 150.5,
        };
        let decoded = Response::from_bytes(response.to_bytes()).expect("ответ корректен");
        assert_eq!(decoded, response);
    }

    #[test]
    fn switched_off_response_survives_a_round_trip() {
        let response = Response {
            is_on: false,
            power: 0.0,
        };
        let decoded = Response::from_bytes(response.to_bytes()).expect("ответ корректен");
        assert_eq!(decoded, response);
    }

    #[test]
    fn response_is_exactly_nine_bytes() {
        let bytes = Response {
            is_on: true,
            power: 1.0,
        }
        .to_bytes();
        assert_eq!(bytes.len(), RESPONSE_LEN);
    }

    #[test]
    fn temperature_survives_a_round_trip() {
        let packet = encode_temperature(-12.25);
        let decoded = decode_temperature(&packet).expect("пакет корректен");
        assert!((decoded - (-12.25)).abs() < f64::EPSILON);
    }

    #[test]
    fn short_temperature_packet_is_rejected() {
        let error = decode_temperature(b"abc").expect_err("три байта — не показание");
        assert_eq!(
            error.to_string(),
            "ошибка протокола: ожидался пакет из 8 байт, получено 3"
        );
    }

    #[test]
    fn long_temperature_packet_is_rejected() {
        assert!(decode_temperature(&[0_u8; 16]).is_err());
    }

    #[test]
    fn invalid_state_byte_is_rejected() {
        let mut bytes = [0_u8; RESPONSE_LEN];
        bytes[0] = 7;
        let error = Response::from_bytes(bytes).expect_err("состояние 7 недопустимо");
        assert_eq!(
            error.to_string(),
            "ошибка протокола: недопустимый байт состояния: 7"
        );
    }
}

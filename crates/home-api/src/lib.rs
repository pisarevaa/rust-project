//! Типы, которыми обмениваются backend-сервис умного дома и его клиенты.
//!
//! Все структуры сериализуются в JSON; перечисления — строками в `snake_case`.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

pub type RoomId = u64;
pub type DeviceId = u64;

/// Максимальная длина названия комнаты или устройства (в символах).
pub const MAX_NAME_LEN: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    Socket,
    Thermometer,
    Light,
}

impl DeviceKind {
    pub const ALL: [Self; 3] = [Self::Socket, Self::Thermometer, Self::Light];
}

impl fmt::Display for DeviceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Socket => "розетка",
            Self::Thermometer => "термометр",
            Self::Light => "светильник",
        })
    }
}

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    On,
    #[default]
    Off,
    Offline,
}

impl DeviceStatus {
    pub const ALL: [Self; 3] = [Self::On, Self::Off, Self::Offline];
}

impl fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::On => "включено",
            Self::Off => "выключено",
            Self::Offline => "не в сети",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    pub id: DeviceId,
    pub room_id: RoomId,
    pub name: String,
    pub kind: DeviceKind,
    pub status: DeviceStatus,
}

/// Краткие сведения о комнате — для списков и отчета.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomSummary {
    pub id: RoomId,
    pub name: String,
    pub device_count: usize,
}

/// Полные сведения о комнате вместе с ее устройствами.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub name: String,
    pub devices: Vec<Device>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewRoom {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewDevice {
    pub name: String,
    pub kind: DeviceKind,
    #[serde(default)]
    pub status: DeviceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseReport {
    pub room_count: usize,
    pub device_count: usize,
    pub devices_by_kind: BTreeMap<DeviceKind, usize>,
    pub devices_by_status: BTreeMap<DeviceStatus, usize>,
    pub rooms: Vec<RoomSummary>,
}

/// Машиночитаемый код ошибки API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    NotFound,
    Validation,
    Conflict,
    Internal,
}

/// Тело ответа backend-сервиса при любой ошибке.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: ErrorCode,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enums_are_snake_case_strings() {
        assert_eq!(
            serde_json::to_string(&DeviceKind::Thermometer).unwrap(),
            "\"thermometer\""
        );
        assert_eq!(
            serde_json::to_string(&ErrorCode::NotFound).unwrap(),
            "\"not_found\""
        );
    }

    #[test]
    fn new_device_status_defaults_to_off() {
        let device: NewDevice =
            serde_json::from_str(r#"{"name": "Лампа", "kind": "light"}"#).unwrap();
        assert_eq!(device.status, DeviceStatus::Off);
    }

    #[test]
    fn report_maps_use_enum_names_as_keys() {
        let report = HouseReport {
            room_count: 0,
            device_count: 1,
            devices_by_kind: BTreeMap::from([(DeviceKind::Socket, 1)]),
            devices_by_status: BTreeMap::from([(DeviceStatus::On, 1)]),
            rooms: Vec::new(),
        };
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["devices_by_kind"]["socket"], 1);
        assert_eq!(json["devices_by_status"]["on"], 1);
    }
}

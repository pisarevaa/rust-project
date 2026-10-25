use std::collections::BTreeMap;

use home_api::{
    Device, DeviceId, DeviceKind, DeviceStatus, HouseReport, NewDevice, NewRoom, Room, RoomId,
    RoomSummary, MAX_NAME_LEN,
};

use crate::error::ApiError;

#[derive(Debug)]
struct RoomEntry {
    name: String,
    devices: BTreeMap<DeviceId, Device>,
}

impl RoomEntry {
    fn summary(&self, id: RoomId) -> RoomSummary {
        RoomSummary {
            id,
            name: self.name.clone(),
            device_count: self.devices.len(),
        }
    }
}

/// Хранилище умного дома в памяти. Идентификаторы выдаются по возрастанию
/// и не переиспользуются после удаления.
#[derive(Debug)]
pub struct Store {
    rooms: BTreeMap<RoomId, RoomEntry>,
    next_room_id: RoomId,
    next_device_id: DeviceId,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            rooms: BTreeMap::new(),
            next_room_id: 1,
            next_device_id: 1,
        }
    }
}

fn validate_name(name: &str, what: &str) -> Result<String, ApiError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(ApiError::Validation(format!(
            "название {what} не может быть пустым"
        )));
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(ApiError::Validation(format!(
            "название {what} длиннее {MAX_NAME_LEN} символов"
        )));
    }
    Ok(name.to_owned())
}

impl Store {
    #[must_use]
    pub fn rooms(&self) -> Vec<RoomSummary> {
        self.rooms
            .iter()
            .map(|(&id, room)| room.summary(id))
            .collect()
    }

    /// # Errors
    /// [`ApiError::RoomNotFound`], если комнаты нет.
    pub fn room(&self, id: RoomId) -> Result<Room, ApiError> {
        let entry = self.entry(id)?;
        Ok(Room {
            id,
            name: entry.name.clone(),
            devices: entry.devices.values().cloned().collect(),
        })
    }

    /// # Errors
    /// [`ApiError::Validation`] для некорректного названия,
    /// [`ApiError::Conflict`], если комната с таким названием уже есть.
    pub fn add_room(&mut self, new: &NewRoom) -> Result<Room, ApiError> {
        let name = validate_name(&new.name, "комнаты")?;
        if self
            .rooms
            .values()
            .any(|room| room.name.to_lowercase() == name.to_lowercase())
        {
            return Err(ApiError::Conflict(format!(
                "комната «{name}» уже существует"
            )));
        }
        let id = self.next_room_id;
        self.next_room_id += 1;
        self.rooms.insert(
            id,
            RoomEntry {
                name: name.clone(),
                devices: BTreeMap::new(),
            },
        );
        Ok(Room {
            id,
            name,
            devices: Vec::new(),
        })
    }

    /// Удаляет комнату вместе со всеми ее устройствами.
    ///
    /// # Errors
    /// [`ApiError::RoomNotFound`], если комнаты нет.
    pub fn remove_room(&mut self, id: RoomId) -> Result<(), ApiError> {
        self.rooms
            .remove(&id)
            .map(|_| ())
            .ok_or(ApiError::RoomNotFound(id))
    }

    /// # Errors
    /// [`ApiError::RoomNotFound`], если комнаты нет.
    pub fn devices(&self, room_id: RoomId) -> Result<Vec<Device>, ApiError> {
        Ok(self.entry(room_id)?.devices.values().cloned().collect())
    }

    /// # Errors
    /// [`ApiError::RoomNotFound`] или [`ApiError::DeviceNotFound`].
    pub fn device(&self, room_id: RoomId, device_id: DeviceId) -> Result<Device, ApiError> {
        self.entry(room_id)?
            .devices
            .get(&device_id)
            .cloned()
            .ok_or(ApiError::DeviceNotFound { room_id, device_id })
    }

    /// # Errors
    /// [`ApiError::RoomNotFound`], если комнаты нет, [`ApiError::Validation`]
    /// для некорректного названия, [`ApiError::Conflict`], если в комнате уже
    /// есть устройство с таким названием.
    pub fn add_device(&mut self, room_id: RoomId, new: &NewDevice) -> Result<Device, ApiError> {
        let name = validate_name(&new.name, "устройства")?;
        let id = self.next_device_id;
        let room = self
            .rooms
            .get_mut(&room_id)
            .ok_or(ApiError::RoomNotFound(room_id))?;
        if room
            .devices
            .values()
            .any(|device| device.name.to_lowercase() == name.to_lowercase())
        {
            return Err(ApiError::Conflict(format!(
                "устройство «{name}» уже есть в комнате «{}»",
                room.name
            )));
        }
        let device = Device {
            id,
            room_id,
            name,
            kind: new.kind,
            status: new.status,
        };
        room.devices.insert(id, device.clone());
        self.next_device_id += 1;
        Ok(device)
    }

    /// # Errors
    /// [`ApiError::RoomNotFound`] или [`ApiError::DeviceNotFound`].
    pub fn remove_device(&mut self, room_id: RoomId, device_id: DeviceId) -> Result<(), ApiError> {
        self.rooms
            .get_mut(&room_id)
            .ok_or(ApiError::RoomNotFound(room_id))?
            .devices
            .remove(&device_id)
            .map(|_| ())
            .ok_or(ApiError::DeviceNotFound { room_id, device_id })
    }

    #[must_use]
    pub fn report(&self) -> HouseReport {
        // Все типы и статусы присутствуют в отчете, даже с нулевым количеством.
        let mut devices_by_kind: BTreeMap<DeviceKind, usize> =
            DeviceKind::ALL.into_iter().map(|kind| (kind, 0)).collect();
        let mut devices_by_status: BTreeMap<DeviceStatus, usize> = DeviceStatus::ALL
            .into_iter()
            .map(|status| (status, 0))
            .collect();
        let devices = self.rooms.values().flat_map(|room| room.devices.values());
        for device in devices {
            *devices_by_kind.entry(device.kind).or_default() += 1;
            *devices_by_status.entry(device.status).or_default() += 1;
        }
        let rooms = self.rooms();
        HouseReport {
            room_count: rooms.len(),
            device_count: rooms.iter().map(|room| room.device_count).sum(),
            devices_by_kind,
            devices_by_status,
            rooms,
        }
    }

    fn entry(&self, id: RoomId) -> Result<&RoomEntry, ApiError> {
        self.rooms.get(&id).ok_or(ApiError::RoomNotFound(id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_room(name: &str) -> NewRoom {
        NewRoom { name: name.into() }
    }

    fn new_device(name: &str, kind: DeviceKind, status: DeviceStatus) -> NewDevice {
        NewDevice {
            name: name.into(),
            kind,
            status,
        }
    }

    #[test]
    fn room_names_are_trimmed_and_unique() {
        let mut store = Store::default();
        let room = store.add_room(&new_room("  Кухня ")).unwrap();
        assert_eq!(room.name, "Кухня");
        assert!(matches!(
            store.add_room(&new_room("кухня")),
            Err(ApiError::Conflict(_))
        ));
        assert!(matches!(
            store.add_room(&new_room("   ")),
            Err(ApiError::Validation(_))
        ));
        assert!(matches!(
            store.add_room(&new_room(&"я".repeat(MAX_NAME_LEN + 1))),
            Err(ApiError::Validation(_))
        ));
    }

    #[test]
    fn device_ids_are_not_reused_after_failed_insert() {
        let mut store = Store::default();
        let room = store.add_room(&new_room("Кухня")).unwrap();
        let first = store
            .add_device(
                room.id,
                &new_device("Чайник", DeviceKind::Socket, DeviceStatus::On),
            )
            .unwrap();
        assert!(store
            .add_device(
                room.id,
                &new_device("Чайник", DeviceKind::Socket, DeviceStatus::On)
            )
            .is_err());
        let second = store
            .add_device(
                room.id,
                &new_device("Лампа", DeviceKind::Light, DeviceStatus::Off),
            )
            .unwrap();
        assert_eq!(second.id, first.id + 1);
    }

    #[test]
    fn report_counts_devices() {
        let mut store = Store::default();
        let kitchen = store.add_room(&new_room("Кухня")).unwrap();
        store.add_room(&new_room("Спальня")).unwrap();
        store
            .add_device(
                kitchen.id,
                &new_device("Чайник", DeviceKind::Socket, DeviceStatus::On),
            )
            .unwrap();
        store
            .add_device(
                kitchen.id,
                &new_device("Термометр", DeviceKind::Thermometer, DeviceStatus::On),
            )
            .unwrap();

        let report = store.report();
        assert_eq!(report.room_count, 2);
        assert_eq!(report.device_count, 2);
        assert_eq!(report.devices_by_kind[&DeviceKind::Socket], 1);
        assert_eq!(report.devices_by_kind[&DeviceKind::Light], 0);
        assert_eq!(report.devices_by_status[&DeviceStatus::On], 2);
        assert_eq!(report.rooms[0].device_count, 2);
        assert_eq!(report.rooms[1].device_count, 0);
    }
}

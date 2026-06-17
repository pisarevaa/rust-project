# Заготовка библиотеки «Умный дом» — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Создать Rust-package с lib-крейтом (типы умного дома) и bin-крейтом (пример использования), проходящий build/clippy/fmt/test.

**Architecture:** lib-крейт делит типы по модулям (термометр, розетка, устройство-enum, комната, дом). «Умное устройство» — это enum `SmartDevice` с вариантами `Thermometer`/`Socket`. Коллекции — `Vec`; доступ по индексу опирается на штатную панику индексации Rust при выходе за границы. bin-крейт собирает дом, печатает отчет, выключает розетку и печатает отчет снова.

**Tech Stack:** Rust (cargo 1.95), стандартная библиотека, без внешних зависимостей. TDD через `cargo test`.

---

## Файловая структура

- Create: `Cargo.toml` — манифест package (lib + bin).
- Create: `src/lib.rs` — объявление модулей и re-export публичных типов.
- Create: `src/thermometer.rs` — `SmartThermometer` + тесты.
- Create: `src/socket.rs` — `SmartSocket` + тесты.
- Create: `src/device.rs` — `enum SmartDevice` + тесты.
- Create: `src/room.rs` — `Room` + тесты.
- Create: `src/house.rs` — `SmartHouse` + тесты.
- Create: `src/main.rs` — bin-крейт, пример использования.

Каждый модуль отвечает за один тип и держит свои тесты в `#[cfg(test)] mod tests`.

---

### Task 1: Каркас package

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/main.rs`

- [ ] **Step 1: Создать `Cargo.toml`**

```toml
[package]
name = "smart-home"
version = "0.1.0"
edition = "2021"

[lib]
name = "smart_home"
path = "src/lib.rs"

[[bin]]
name = "smart-home"
path = "src/main.rs"
```

- [ ] **Step 2: Создать `src/lib.rs` с пустыми модулями**

```rust
pub mod device;
pub mod house;
pub mod room;
pub mod socket;
pub mod thermometer;
```

- [ ] **Step 3: Создать заглушки модулей, чтобы package компилировался**

Создать пустые файлы `src/thermometer.rs`, `src/socket.rs`, `src/device.rs`,
`src/room.rs`, `src/house.rs` (по одной строке-комментарию `// заполняется далее`
в каждом — пустой модуль валиден).

- [ ] **Step 4: Создать минимальный `src/main.rs`**

```rust
fn main() {
    println!("smart-home");
}
```

- [ ] **Step 5: Проверить сборку**

Run: `cargo build`
Expected: успешная сборка, без ошибок.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/
git commit -m "chore: каркас package smart-home (lib + bin)"
```

---

### Task 2: SmartThermometer

**Files:**
- Modify: `src/thermometer.rs`

- [ ] **Step 1: Написать падающий тест**

В `src/thermometer.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_constructed_temperature() {
        let t = SmartThermometer::new("Гостиная".to_string(), 21.5);
        assert_eq!(t.temperature(), 21.5);
    }
}
```

- [ ] **Step 2: Запустить тест — убедиться, что не компилируется/падает**

Run: `cargo test --lib thermometer`
Expected: ошибка компиляции — `SmartThermometer` не определен.

- [ ] **Step 3: Реализовать тип**

В начало `src/thermometer.rs` (перед блоком тестов):

```rust
pub struct SmartThermometer {
    name: String,
    temperature: f64,
}

impl SmartThermometer {
    pub fn new(name: String, temperature: f64) -> Self {
        Self { name, temperature }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn temperature(&self) -> f64 {
        self.temperature
    }
}
```

- [ ] **Step 4: Запустить тест**

Run: `cargo test --lib thermometer`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/thermometer.rs
git commit -m "feat: тип SmartThermometer"
```

---

### Task 3: SmartSocket

**Files:**
- Modify: `src/socket.rs`

- [ ] **Step 1: Написать падающие тесты**

В `src/socket.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_socket_is_off_with_zero_power() {
        let s = SmartSocket::new("Розетка".to_string(), 100.0);
        assert!(!s.is_on());
        assert_eq!(s.current_power(), 0.0);
    }

    #[test]
    fn turn_on_reports_power_and_state() {
        let mut s = SmartSocket::new("Розетка".to_string(), 100.0);
        s.turn_on();
        assert!(s.is_on());
        assert_eq!(s.current_power(), 100.0);
    }

    #[test]
    fn turn_off_resets_power() {
        let mut s = SmartSocket::new("Розетка".to_string(), 100.0);
        s.turn_on();
        s.turn_off();
        assert!(!s.is_on());
        assert_eq!(s.current_power(), 0.0);
    }
}
```

- [ ] **Step 2: Запустить тесты — убедиться, что не компилируется**

Run: `cargo test --lib socket`
Expected: ошибка компиляции — `SmartSocket` не определен.

- [ ] **Step 3: Реализовать тип**

В начало `src/socket.rs`:

```rust
pub struct SmartSocket {
    name: String,
    is_on: bool,
    power: f64,
}

impl SmartSocket {
    pub fn new(name: String, power: f64) -> Self {
        Self {
            name,
            is_on: false,
            power,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn turn_on(&mut self) {
        self.is_on = true;
    }

    pub fn turn_off(&mut self) {
        self.is_on = false;
    }

    pub fn is_on(&self) -> bool {
        self.is_on
    }

    pub fn current_power(&self) -> f64 {
        if self.is_on {
            self.power
        } else {
            0.0
        }
    }
}
```

- [ ] **Step 4: Запустить тесты**

Run: `cargo test --lib socket`
Expected: PASS (3 теста).

- [ ] **Step 5: Commit**

```bash
git add src/socket.rs
git commit -m "feat: тип SmartSocket"
```

---

### Task 4: SmartDevice (enum)

**Files:**
- Modify: `src/device.rs`

- [ ] **Step 1: Написать падающий тест**

В `src/device.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::socket::SmartSocket;
    use crate::thermometer::SmartThermometer;

    #[test]
    fn wraps_thermometer() {
        let d = SmartDevice::Thermometer(SmartThermometer::new("Т".to_string(), 20.0));
        // report не паникует и доступен match по варианту
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
```

- [ ] **Step 2: Запустить тест — убедиться, что не компилируется**

Run: `cargo test --lib device`
Expected: ошибка компиляции — `SmartDevice` не определен.

- [ ] **Step 3: Реализовать enum**

В начало `src/device.rs`:

```rust
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
                let state = if s.is_on() { "включена" } else { "выключена" };
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
```

- [ ] **Step 4: Запустить тест**

Run: `cargo test --lib device`
Expected: PASS (2 теста; в выводе видны строки отчета).

- [ ] **Step 5: Commit**

```bash
git add src/device.rs
git commit -m "feat: enum SmartDevice с отчетом о состоянии"
```

---

### Task 5: Room

**Files:**
- Modify: `src/room.rs`

- [ ] **Step 1: Написать падающие тесты**

В `src/room.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::device::SmartDevice;
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
```

- [ ] **Step 2: Запустить тесты — убедиться, что не компилируется**

Run: `cargo test --lib room`
Expected: ошибка компиляции — `Room` не определен.

- [ ] **Step 3: Реализовать тип**

В начало `src/room.rs`:

```rust
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
```

- [ ] **Step 4: Запустить тесты**

Run: `cargo test --lib room`
Expected: PASS (3 теста).

- [ ] **Step 5: Commit**

```bash
git add src/room.rs
git commit -m "feat: тип Room с доступом по индексу и отчетом"
```

---

### Task 6: SmartHouse

**Files:**
- Modify: `src/house.rs`

- [ ] **Step 1: Написать падающие тесты**

В `src/house.rs`:

```rust
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
```

- [ ] **Step 2: Запустить тесты — убедиться, что не компилируется**

Run: `cargo test --lib house`
Expected: ошибка компиляции — `SmartHouse` не определен.

- [ ] **Step 3: Реализовать тип**

В начало `src/house.rs`:

```rust
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
```

- [ ] **Step 4: Запустить тесты**

Run: `cargo test --lib house`
Expected: PASS (3 теста).

- [ ] **Step 5: Commit**

```bash
git add src/house.rs
git commit -m "feat: тип SmartHouse с доступом по индексу и отчетом"
```

---

### Task 7: Пример использования (bin)

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Реализовать сценарий примера**

Заменить содержимое `src/main.rs`:

```rust
use smart_home::device::SmartDevice;
use smart_home::house::SmartHouse;
use smart_home::room::Room;
use smart_home::socket::SmartSocket;
use smart_home::thermometer::SmartThermometer;

fn main() {
    let mut house = SmartHouse::new(
        "Мой дом".to_string(),
        vec![
            Room::new(
                "Гостиная".to_string(),
                vec![
                    SmartDevice::Thermometer(SmartThermometer::new(
                        "Термометр гостиной".to_string(),
                        21.5,
                    )),
                    SmartDevice::Socket({
                        let mut s = SmartSocket::new("Розетка гостиной".to_string(), 150.0);
                        s.turn_on();
                        s
                    }),
                ],
            ),
            Room::new(
                "Кухня".to_string(),
                vec![
                    SmartDevice::Thermometer(SmartThermometer::new(
                        "Термометр кухни".to_string(),
                        24.0,
                    )),
                    SmartDevice::Socket({
                        let mut s = SmartSocket::new("Розетка кухни".to_string(), 200.0);
                        s.turn_on();
                        s
                    }),
                ],
            ),
        ],
    );

    println!("Первичный отчет:");
    house.report();

    // Выключаем розетку в гостиной (комната 0, устройство 1).
    if let SmartDevice::Socket(socket) = house.room_mut(0).device_mut(1) {
        socket.turn_off();
    }

    println!();
    println!("Отчет после выключения розетки в гостиной:");
    house.report();
}
```

- [ ] **Step 2: Запустить пример**

Run: `cargo run`
Expected: два отчета; во втором у «Розетки гостиной» состояние «выключена» и мощность `0.0`.

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: пример использования умного дома"
```

---

### Task 8: Финальная проверка качества

**Files:** нет изменений кода (правки только при наличии замечаний).

- [ ] **Step 1: clippy**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: без предупреждений и ошибок. Если есть — исправить и закоммитить.

- [ ] **Step 2: fmt**

Run: `cargo fmt --check`
Expected: без замечаний. Если есть — выполнить `cargo fmt` и закоммитить.

- [ ] **Step 3: Полный прогон тестов**

Run: `cargo test`
Expected: все тесты проходят (термометр, розетка ×3, устройство ×2, комната ×3, дом ×3).

- [ ] **Step 4: Сборка**

Run: `cargo build`
Expected: успешно.

- [ ] **Step 5: Commit при необходимости**

```bash
git add -A
git commit -m "chore: чистка clippy/fmt" || echo "нет изменений"
```

---

## Self-Review

**Покрытие спецификации:**
- SmartThermometer (конструктор, температура) → Task 2. ✓
- SmartSocket (конструктор, вкл/выкл, состояние, мощность 0 при выкл) → Task 3. ✓
- SmartDevice (enum, отчет в stdout) → Task 4. ✓
- Room (конструктор, `device`/`device_mut` по индексу, отчет) → Task 5. ✓
- SmartHouse (конструктор, `room`/`room_mut` по индексу, отчет) → Task 6. ✓
- Паника при выходе за границы индекса → `#[should_panic]` в Task 5 и Task 6. ✓
- Пример: создать дом, отчет, выключить розетку, отчет снова → Task 7. ✓
- lib + bin в одном package → Task 1. ✓
- clippy / fmt / тесты → Task 8 + тесты в каждой задаче. ✓

**Заглушки:** не обнаружено — весь код приведен полностью.

**Согласованность типов:** имена методов едины во всех задачах
(`new`, `name`, `temperature`, `turn_on`, `turn_off`, `is_on`, `current_power`,
`report`, `device`, `device_mut`, `room`, `room_mut`). Пути модулей в `use`
(`smart_home::...` в bin, `crate::...` в lib) согласованы.

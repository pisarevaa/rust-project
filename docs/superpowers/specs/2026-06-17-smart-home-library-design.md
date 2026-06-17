# Дизайн: заготовка библиотеки «Умный дом»

Дата: 2026-06-17

## Цель

Создать заготовку библиотеки «Умный дом» в виде Rust-package, содержащего
lib-крейт (сами типы) и bin-крейт (пример использования). Пример демонстрирует
функционал: создает дом, печатает отчет, выключает розетку и печатает отчет снова.

## Структура package

Один package, два крейта (lib + bin):

```
smart-home/
├── Cargo.toml
├── src/
│   ├── lib.rs          # объявление модулей, re-export публичных типов
│   ├── thermometer.rs  # SmartThermometer
│   ├── socket.rs       # SmartSocket
│   ├── device.rs       # enum SmartDevice
│   ├── room.rs         # Room
│   ├── house.rs        # SmartHouse
│   └── main.rs         # bin-крейт: пример использования
```

## Типы и публичный API

### SmartThermometer (`thermometer.rs`)

Поля:
- `name: String`
- `temperature: f64`

Методы:
- `new(name: String, temperature: f64) -> Self` — конструктор, принимающий значения полей.
- `temperature(&self) -> f64` — возвращает текущую температуру (значение поля).

### SmartSocket (`socket.rs`)

Поля:
- `name: String`
- `is_on: bool`
- `power: f64` — мощность, отдаваемая во включенном состоянии.

Методы:
- `new(name: String, power: f64) -> Self` — конструктор. Начальное состояние `is_on = false`.
- `turn_on(&mut self)` — включить.
- `turn_off(&mut self)` — выключить.
- `is_on(&self) -> bool` — текущее состояние.
- `current_power(&self) -> f64` — `0.0` если выключена, иначе значение поля `power`.

### SmartDevice (`device.rs`)

Enum, содержащий одно из устройств:

```rust
enum SmartDevice {
    Thermometer(SmartThermometer),
    Socket(SmartSocket),
}
```

Методы:
- `report(&self)` — печатает в stdout сообщение о состоянии устройства
  через `match` по варианту (температура для термометра; состояние и мощность
  для розетки).

### Room (`room.rs`)

Поля:
- `name: String`
- `devices: Vec<SmartDevice>`

Методы:
- `new(name: String, devices: Vec<SmartDevice>) -> Self` — конструктор.
- `device(&self, index: usize) -> &SmartDevice` — ссылка на устройство по индексу.
- `device_mut(&mut self, index: usize) -> &mut SmartDevice` — мутабельная ссылка.
- `report(&self)` — печатает отчет обо всех устройствах в комнате.

### SmartHouse (`house.rs`)

Поля:
- `name: String`
- `rooms: Vec<Room>`

Методы:
- `new(name: String, rooms: Vec<Room>) -> Self` — конструктор.
- `room(&self, index: usize) -> &Room` — ссылка на комнату по индексу.
- `room_mut(&mut self, index: usize) -> &mut Room` — мутабельная ссылка.
- `report(&self)` — печатает отчет обо всех комнатах.

## Ключевые решения

- **`Vec` вместо фиксированных массивов.** Идиоматично для Rust; размеры
  коллекций задание разрешает выбирать произвольно.
- **«Произвольное число» = значение поля из конструктора**, не `rand`.
  Делает модульные тесты детерминированными и не добавляет внешних зависимостей.
  Температура термометра и мощность розетки задаются при создании.
- **Доступ по индексу через индексацию `Vec`/среза** (`&self.devices[index]`).
  Стандартная индексация Rust сама вызывает `panic!` при выходе за границы —
  это ровно соответствует требованию «приложение должно аварийно завершаться».
  Отдельный ручной `panic!()` не нужен.
- **Отчеты** реализованы методами `report()`, печатающими через `println!`.

## Пример использования (`main.rs`)

1. Создать дом: 2 комнаты, в каждой по термометру и розетке; розетки включены.
2. Вывести отчет дома (`house.report()`).
3. Выключить розетку в одной из комнат через мутабельный доступ:
   `house.room_mut(i).device_mut(j)` → `match` по варианту `SmartDevice::Socket`
   → `turn_off()`.
4. Снова вывести отчет — в выводе видно изменение мощности розетки (была N → 0).

## Тестирование

Модульные тесты в каждом модуле в блоках `#[cfg(test)] mod tests`:

- `SmartThermometer` возвращает заданную температуру.
- `SmartSocket`: после `turn_on` состояние включено и `current_power` равно `power`;
  после `turn_off` состояние выключено и `current_power` равно `0.0`.
- `Room`: `device`/`device_mut` возвращают корректное устройство по индексу.
- `SmartHouse`: `room`/`room_mut` возвращают корректную комнату по индексу.
- `#[should_panic]`-тесты на выход за границы индекса для `Room` и `SmartHouse`.

## Критерии приемки

- `cargo build` собирается без ошибок.
- `cargo run` выполняет пример и печатает отчет о доме (дважды, с разницей).
- `cargo clippy` — без предупреждений и ошибок.
- `cargo fmt --check` — без замечаний.
- `cargo test` — все модульные тесты проходят.

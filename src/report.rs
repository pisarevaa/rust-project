//! Отчеты об объектах умного дома и компоновщик для их объединения.

pub trait Report {
    fn report(&self) -> String;
}

impl<T: Report + ?Sized> Report for &T {
    fn report(&self) -> String {
        (**self).report()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Empty;

impl Report for Empty {
    fn report(&self) -> String {
        String::new()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Chain<H, T> {
    head: H,
    tail: T,
}

impl<H: Report, T: Report> Report for Chain<H, T> {
    fn report(&self) -> String {
        let head = self.head.report();
        let tail = self.tail.report();
        match (head.is_empty(), tail.is_empty()) {
            (true, _) => tail,
            (_, true) => head,
            _ => format!("{head}\n{tail}"),
        }
    }
}

/// Компоновщик отчетов: собирает произвольные объекты, умеющие отчитываться,
/// и печатает общий отчет.
///
/// ```
/// use smart_home::report::Reporter;
/// use smart_home::socket::SmartSocket;
/// use smart_home::thermometer::SmartThermometer;
///
/// let socket = SmartSocket::mock(60.0);
/// let thermometer = SmartThermometer::mock(21.5);
///
/// Reporter::new().add(&socket).add(&thermometer).report();
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct Reporter<T> {
    items: T,
}

impl Reporter<Empty> {
    #[must_use]
    pub fn new() -> Self {
        Self { items: Empty }
    }
}

impl<T: Report> Reporter<T> {
    #[allow(clippy::should_implement_trait)]
    #[must_use]
    pub fn add<R: Report + ?Sized>(self, item: &R) -> Reporter<Chain<T, &R>> {
        Reporter {
            items: Chain {
                head: self.items,
                tail: item,
            },
        }
    }

    #[must_use]
    pub fn rendered(&self) -> String {
        self.items.report()
    }

    pub fn report(&self) {
        println!("{}", self.rendered());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room::Room;
    use crate::socket::SmartSocket;
    use crate::thermometer::SmartThermometer;
    use crate::{device::SmartDevice, room};

    #[test]
    fn empty_reporter_renders_nothing() {
        assert_eq!(Reporter::new().rendered(), "");
    }

    #[test]
    fn single_item_renders_its_own_report() {
        let socket = SmartSocket::mock(60.0);
        let device: SmartDevice = socket.into();
        assert_eq!(
            Reporter::new().add(&device).rendered(),
            "розетка выключена, мощность 0.0 Вт"
        );
    }

    #[test]
    fn items_are_rendered_in_the_order_they_were_added() {
        let first: SmartDevice = SmartThermometer::mock(21.5).into();
        let second: SmartDevice = SmartSocket::mock(60.0).into();
        assert_eq!(
            Reporter::new().add(&first).add(&second).rendered(),
            "термометр показывает 21.5 °C\nрозетка выключена, мощность 0.0 Вт"
        );
    }

    #[test]
    fn objects_of_different_types_go_into_one_reporter() {
        let device: SmartDevice = SmartSocket::mock(60.0).into();
        let room = room! {
            "Термометр" => SmartThermometer::mock(24.0),
        };
        let mut house = crate::house::SmartHouse::new("Дом");
        house.add_room("Кладовка", Room::new());

        assert_eq!(
            Reporter::new()
                .add(&device)
                .add(&room)
                .add(&house)
                .rendered(),
            concat!(
                "розетка выключена, мощность 0.0 Вт\n",
                "  Термометр: термометр показывает 24.0 °C\n",
                "=== Отчет о доме «Дом» ===\n",
                "Комната «Кладовка»:"
            )
        );
    }

    #[test]
    fn empty_reports_do_not_leave_blank_lines() {
        let empty = Room::new();
        let device: SmartDevice = SmartSocket::mock(60.0).into();
        assert_eq!(
            Reporter::new()
                .add(&empty)
                .add(&device)
                .add(&empty)
                .rendered(),
            "розетка выключена, мощность 0.0 Вт"
        );
    }

    #[test]
    fn reference_forwards_the_report_of_the_referenced_value() {
        let room = room! {
            "Розетка" => SmartSocket::mock(60.0),
        };
        let reference = &room;
        assert_eq!(reference.report(), room.report());
    }
}

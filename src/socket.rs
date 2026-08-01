#[derive(Debug)]
pub struct SmartSocket {
    is_on: bool,
    power: f64,
}

impl SmartSocket {
    #[must_use]
    pub fn new(power: f64) -> Self {
        Self {
            is_on: false,
            power,
        }
    }

    pub fn turn_on(&mut self) {
        self.is_on = true;
    }

    pub fn turn_off(&mut self) {
        self.is_on = false;
    }

    #[must_use]
    pub fn is_on(&self) -> bool {
        self.is_on
    }

    #[must_use]
    pub fn current_power(&self) -> f64 {
        if self.is_on {
            self.power
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_socket_is_off_with_zero_power() {
        let s = SmartSocket::new(100.0);
        assert!(!s.is_on());
        assert!((s.current_power() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn turn_on_reports_power_and_state() {
        let mut s = SmartSocket::new(100.0);
        s.turn_on();
        assert!(s.is_on());
        assert!((s.current_power() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn turn_off_resets_power() {
        let mut s = SmartSocket::new(100.0);
        s.turn_on();
        s.turn_off();
        assert!(!s.is_on());
        assert!((s.current_power() - 0.0).abs() < f64::EPSILON);
    }
}

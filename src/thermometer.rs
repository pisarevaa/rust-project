#[derive(Debug)]
pub struct SmartThermometer {
    temperature: f64,
}

impl SmartThermometer {
    #[must_use]
    pub fn new(temperature: f64) -> Self {
        Self { temperature }
    }

    #[must_use]
    pub fn temperature(&self) -> f64 {
        self.temperature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_constructed_temperature() {
        let t = SmartThermometer::new(21.5);
        assert!((t.temperature() - 21.5).abs() < f64::EPSILON);
    }
}

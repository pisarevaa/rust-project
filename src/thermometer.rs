pub struct SmartThermometer {
    name: String,
    temperature: f64,
}

impl SmartThermometer {
    #[must_use]
    pub fn new(name: String, temperature: f64) -> Self {
        Self { name, temperature }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
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
        let t = SmartThermometer::new("Гостиная".to_string(), 21.5);
        assert!((t.temperature() - 21.5).abs() < f64::EPSILON);
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_constructed_temperature() {
        let t = SmartThermometer::new("Гостиная".to_string(), 21.5);
        assert_eq!(t.temperature(), 21.5);
    }
}

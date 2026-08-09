/// Конфигурация имитатора термометра.

use std::fmt;
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulatorConfig {
    pub address: String,
    pub period: Duration,
}

impl SimulatorConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        Self::parse(&fs::read_to_string(path)?)
    }

    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        let mut address = None;
        let mut period = None;

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| ConfigError::Syntax(line.to_string()))?;
            let (key, value) = (key.trim(), value.trim());

            match key {
                "address" => {
                    if value.is_empty() {
                        return Err(ConfigError::InvalidValue {
                            key: key.to_string(),
                            value: value.to_string(),
                        });
                    }
                    address = Some(value.to_string());
                }
                "period_ms" => {
                    let millis: u64 = value.parse().map_err(|_| ConfigError::InvalidValue {
                        key: key.to_string(),
                        value: value.to_string(),
                    })?;
                    if millis == 0 {
                        return Err(ConfigError::InvalidValue {
                            key: key.to_string(),
                            value: value.to_string(),
                        });
                    }
                    period = Some(Duration::from_millis(millis));
                }
                other => return Err(ConfigError::UnknownKey(other.to_string())),
            }
        }

        Ok(Self {
            address: address.ok_or(ConfigError::MissingKey("address"))?,
            period: period.ok_or(ConfigError::MissingKey("period_ms"))?,
        })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Syntax(String),
    UnknownKey(String),
    InvalidValue { key: String, value: String },
    MissingKey(&'static str),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "не удалось прочитать файл конфигурации: {e}"),
            Self::Syntax(line) => write!(f, "строка не содержит знака «=»: «{line}»"),
            Self::UnknownKey(key) => write!(f, "неизвестный ключ «{key}»"),
            Self::InvalidValue { key, value } => {
                write!(f, "недопустимое значение ключа «{key}»: «{value}»")
            }
            Self::MissingKey(key) => write!(f, "в конфигурации отсутствует ключ «{key}»"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_minimal_config() {
        let config = SimulatorConfig::parse("address = 127.0.0.1:8082\nperiod_ms = 500")
            .expect("конфигурация корректна");
        assert_eq!(
            config,
            SimulatorConfig {
                address: "127.0.0.1:8082".to_string(),
                period: Duration::from_millis(500),
            }
        );
    }

    #[test]
    fn ignores_comments_and_blank_lines() {
        let config = SimulatorConfig::parse(
            "# адрес\n\naddress = 127.0.0.1:8082\n\n  # период\nperiod_ms = 250\n",
        )
        .expect("конфигурация корректна");
        assert_eq!(config.period, Duration::from_millis(250));
    }

    #[test]
    fn tolerates_missing_spaces_around_the_equals_sign() {
        let config = SimulatorConfig::parse("address=127.0.0.1:8082\nperiod_ms=100")
            .expect("конфигурация корректна");
        assert_eq!(config.address, "127.0.0.1:8082");
    }

    #[test]
    fn reports_a_line_without_an_equals_sign() {
        let error = SimulatorConfig::parse("address 127.0.0.1:8082").expect_err("строка битая");
        assert_eq!(
            error.to_string(),
            "строка не содержит знака «=»: «address 127.0.0.1:8082»"
        );
    }

    #[test]
    fn reports_an_unknown_key() {
        let error = SimulatorConfig::parse("port = 8082").expect_err("ключ неизвестен");
        assert_eq!(error.to_string(), "неизвестный ключ «port»");
    }

    #[test]
    fn reports_a_non_numeric_period() {
        let error = SimulatorConfig::parse("address = 127.0.0.1:8082\nperiod_ms = быстро")
            .expect_err("период не число");
        assert_eq!(
            error.to_string(),
            "недопустимое значение ключа «period_ms»: «быстро»"
        );
    }

    #[test]
    fn reports_a_zero_period() {
        let error = SimulatorConfig::parse("address = 127.0.0.1:8082\nperiod_ms = 0")
            .expect_err("нулевой период бессмыслен");
        assert!(matches!(error, ConfigError::InvalidValue { .. }));
    }

    #[test]
    fn reports_an_empty_address() {
        let error = SimulatorConfig::parse("address =\nperiod_ms = 100").expect_err("адрес пустой");
        assert!(matches!(error, ConfigError::InvalidValue { .. }));
    }

    #[test]
    fn reports_a_missing_address() {
        let error = SimulatorConfig::parse("period_ms = 100").expect_err("адреса нет");
        assert_eq!(
            error.to_string(),
            "в конфигурации отсутствует ключ «address»"
        );
    }

    #[test]
    fn reports_a_missing_period() {
        let error = SimulatorConfig::parse("address = 127.0.0.1:8082").expect_err("периода нет");
        assert_eq!(
            error.to_string(),
            "в конфигурации отсутствует ключ «period_ms»"
        );
    }

    #[test]
    fn reports_a_missing_file() {
        let error =
            SimulatorConfig::load("нет-такого-файла.conf").expect_err("файла не существует");
        assert!(matches!(error, ConfigError::Io(_)));
    }
}

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::DriverError::{
    AboveAllowedAlcoholLevel, LicenceExpired, UnderRequiredAge, WithoutLicence,
};

pub trait Validator<T, E> {
    fn validate(&self, t: &T) -> Vec<E>;
}

pub struct DriverValidator {
    rules: Vec<Box<dyn Rule<Driver, DriverError>>>,
}

impl DriverValidator {
    pub fn new(rules: Vec<Box<dyn Rule<Driver, DriverError>>>) -> Self {
        Self { rules }
    }
}

impl Validator<Driver, DriverError> for DriverValidator {
    fn validate(&self, driver: &Driver) -> Vec<DriverError> {
        let (_, errors): (Vec<_>, Vec<_>) = self
            .rules
            .iter()
            .map(|rule| rule.run(driver))
            .partition(Result::is_ok);
        errors.into_iter().map(Result::unwrap_err).collect()
    }
}

pub struct DriverValidatorBuilder {
    rules: Vec<Box<dyn Rule<Driver, DriverError>>>,
}

impl DriverValidatorBuilder {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn with_rule(mut self, rule: Box<dyn Rule<Driver, DriverError>>) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn build(self) -> DriverValidator {
        DriverValidator { rules: self.rules }
    }
}

pub trait Rule<T, E> {
    fn run(&self, t: &T) -> Result<(), E>;
}

pub struct Driver {
    pub age: u8,
    pub alcohol_in_blood: f32,
    pub licence: Option<Licence>,
}

#[derive(Copy, Clone)]
pub struct Licence {
    pub licence_type: LicenceType,
    pub expiration: DateTime<Utc>,
}

#[derive(Copy, Clone)]
pub enum LicenceType {
    A,
    A1,
    B,
    C,
    D,
    BE,
    CE,
    DE,
}

impl Licence {
    pub fn is_valid_in_date(&self, date: DateTime<Utc>) -> bool {
        self.expiration >= date
    }
}

pub struct IsSober {
    allowed_level: f32,
}

impl Rule<Driver, DriverError> for IsSober {
    fn run(&self, driver: &Driver) -> Result<(), DriverError> {
        if driver.alcohol_in_blood > self.allowed_level {
            return Err(AboveAllowedAlcoholLevel(driver.alcohol_in_blood));
        }
        Ok(())
    }
}

pub struct HasAge {
    required_age: u8,
}

impl Rule<Driver, DriverError> for HasAge {
    fn run(&self, driver: &Driver) -> Result<(), DriverError> {
        if driver.age < self.required_age {
            return Err(UnderRequiredAge(driver.age));
        }
        Ok(())
    }
}

pub struct HasDrivingLicence;

impl Rule<Driver, DriverError> for HasDrivingLicence {
    fn run(&self, driver: &Driver) -> Result<(), DriverError> {
        driver.licence.map_or(Err(WithoutLicence), |_| Ok(()))
    }
}

pub struct HasValidDrivingLicence {
    date: DateTime<Utc>,
}

impl Rule<Driver, DriverError> for HasValidDrivingLicence {
    fn run(&self, driver: &Driver) -> Result<(), DriverError> {
        driver.licence.map_or(Ok(()), |licence| {
            if !licence.is_valid_in_date(self.date) {
                return Err(LicenceExpired(licence.expiration));
            }
            Ok(())
        })
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum DriverError {
    #[error("Alcohol level is: {} grams/lt ", .0)]
    AboveAllowedAlcoholLevel(f32),
    #[error("Age is: {} years", .0)]
    UnderRequiredAge(u8),
    #[error("Without licence")]
    WithoutLicence,
    #[error("Licence expired on date: {}", .0)]
    LicenceExpired(DateTime<Utc>),
}

fn main() {}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use crate::DriverError::{
        AboveAllowedAlcoholLevel, LicenceExpired, UnderRequiredAge, WithoutLicence,
    };
    use crate::LicenceType::A;

    use super::*;

    #[test]
    pub fn driver_should_not_be_under_minimum_age() {
        let driver = Driver {
            age: 17,
            alcohol_in_blood: 0.4,
            licence: Some(Licence {
                licence_type: A,
                expiration: Utc::now(),
            }),
        };

        let rule = HasAge { required_age: 18 };

        let result = rule.run(&driver);

        match result {
            Ok(_) => panic!("should not happen"),
            Err(e) => assert_eq!(UnderRequiredAge(17), e),
        }
    }

    #[test]
    pub fn driver_should_be_sober() {
        let driver = Driver {
            age: 18,
            alcohol_in_blood: 0.5,
            licence: Some(Licence {
                licence_type: A,
                expiration: Utc::now(),
            }),
        };

        let rule = IsSober {
            allowed_level: 0.49,
        };

        let result = rule.run(&driver);

        match result {
            Ok(_) => panic!("should not happen"),
            Err(e) => assert_eq!(AboveAllowedAlcoholLevel(0.5), e),
        }
    }

    #[test]
    pub fn driver_should_have_licence() {
        let driver = Driver {
            age: 18,
            alcohol_in_blood: 0.0,
            licence: None,
        };

        let rule = HasDrivingLicence;

        let result = rule.run(&driver);

        match result {
            Ok(_) => panic!("should not happen"),
            Err(e) => assert_eq!(WithoutLicence, e),
        }
    }

    #[test]
    pub fn driver_should_have_valid_licence() {
        let today = Utc::now();
        let expiration_date = today - Duration::days(1);
        let driver = Driver {
            age: 18,
            alcohol_in_blood: 0.0,
            licence: Some(Licence {
                licence_type: A,
                expiration: expiration_date,
            }),
        };

        let rule = HasValidDrivingLicence {
            date: today,
        };

        let result = rule.run(&driver);

        match result {
            Ok(_) => panic!("should not happen"),
            Err(e) => assert_eq!(LicenceExpired(expiration_date), e),
        }
    }

    #[test]
    pub fn validator_runs_multiple_rules_and_returns_errors() {
        let driver = Driver {
            age: 17,
            alcohol_in_blood: 0.3,
            licence: None,
        };

        let validator = DriverValidatorBuilder::new()
            .with_rule(Box::new(HasDrivingLicence))
            .with_rule(Box::new(IsSober {
                allowed_level: 0.49,
            }))
            .with_rule(Box::new(HasAge { required_age: 18 }))
            .build();

        let result = validator.validate(&driver);

        assert_eq!(result.len(), 2)
    }
}

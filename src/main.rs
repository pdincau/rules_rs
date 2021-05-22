use thiserror::Error;
use crate::DriverError::{AboveAllowedAlcoholLevel, UnderRequiredAge};

pub trait Rule<T, E> {
    fn run(&self, t: T) -> Result<(), E>;
}


struct Driver {
    pub age: u8,
    pub alcohol_in_blood: f32,
}

pub struct IsSober {
    allowed_level: f32,
}

impl Rule<Driver, DriverError> for IsSober {

    fn run(&self, driver: Driver) -> Result<(), DriverError> {
        if driver.alcohol_in_blood > self.allowed_level {
            return Err(AboveAllowedAlcoholLevel(driver.alcohol_in_blood))
        }
        Ok(())
    }
}

pub struct HasAge {
    required_age: u8,
}

impl Rule<Driver, DriverError> for HasAge {

    fn run(&self, driver: Driver) -> Result<(), DriverError> {
        if driver.age < self.required_age {
            return Err(UnderRequiredAge(driver.age))
        }
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum DriverError {
    #[error("Alcohol level is: {} grams/lt ", .0)]
    AboveAllowedAlcoholLevel(f32),
    #[error("Age is: {} years", .0)]
    UnderRequiredAge(u8),
}

fn main() {
}

#[test]
pub fn driver_should_not_be_under_minimum_age() {
    let driver = Driver { age: 17, alcohol_in_blood: 0.4 };

    let rule = HasAge { required_age: 18 };

    let result = rule.run(driver);

    match result {
        Ok(_) => panic!("should not happen"),
        Err(e) => assert_eq!(UnderRequiredAge(17), e)
    }
}

#[test]
pub fn driver_should_be_sober() {
    let driver = Driver { age: 18, alcohol_in_blood: 0.5 };

    let rule = IsSober { allowed_level: 0.49};

    let result = rule.run(driver);

    match result {
        Ok(_) => panic!("should not happen"),
        Err(e) => assert_eq!(AboveAllowedAlcoholLevel(0.5), e)
    }
}
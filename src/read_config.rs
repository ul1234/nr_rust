use crate::err::Error;
use serde::de::DeserializeOwned;
use std::fs::File;

pub fn load_config<T: DeserializeOwned>(file_name: &str) -> Result<T, Error> {
    let f = File::open(file_name)?;
    let value: T = serde_json::from_reader(f).unwrap();
    Ok(value)
}

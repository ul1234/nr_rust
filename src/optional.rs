use serde::ser::{Serialize, Serializer};
use serde::de::{Deserialize, Deserializer};
use crate::err::Error;

#[derive(Debug)]
pub struct Optional<T> {
    val: Option<T>,
    possible_val: Option<Vec<T>>,
    default_val: Option<T>,
}

impl<T: PartialEq> Optional<T> {
    pub fn check(&self) -> Result<(), Error> {
        if let Some(val) = &self.val {
            if let Some(vec) = &self.possible_val {
                return if vec.contains(val) { Ok(()) } else { Err("check fail")? }
            }
        }
        Ok(())
    }

    pub fn new(val: T) -> Self {
        Self {
            val: Some(val),
            possible_val: None,
            default_val: None,
        }
    }

    pub fn possible(&mut self, vec: Vec<T>) -> &mut Self {
        self.possible_val = Some(vec);
        self
    }

    pub fn default(&mut self, val: T) -> &mut Self {
        self.default_val = Some(val);
        self
    }
}

// https://serde.rs/impl-serialize.html
impl<T: Serialize> Serialize for Optional<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.val {
            Some(val) => serializer.serialize_some(val),
            None => serializer.serialize_none(),
        }
    }
}

// https://serde.rs/custom-date-format.html
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Optional<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let val = Option::deserialize(deserializer)?;
        Ok(Self {
            val,
            possible_val: None,
            default_val: None,
        })
    }
}

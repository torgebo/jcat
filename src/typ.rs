//! Common type
use serde::{Deserialize, Serialize};

/// Data holds values of type `T`.
///
/// For arbitrary JSON objects, use `serde_json::Value`.
/// For reading serialized types, copy this definition
/// and compile for your value of `T`.
#[derive(Serialize, Deserialize, Debug)]
pub struct Data<T>
where
    T: 'static,
{
    pub data: Vec<T>,
}

impl<T> Default for Data<T> {
    fn default() -> Self {
        Self {
            data: Vec::<_>::with_capacity(16394),
        }
    }
}

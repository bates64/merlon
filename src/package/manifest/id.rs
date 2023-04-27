use std::{ops::Deref, str::FromStr};
use std::fmt;
use uuid::Uuid;
use pyo3::{prelude::*, exceptions::PyValueError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Id(Uuid);

impl Id {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl From<Uuid> for Id {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<Id> for Uuid {
    fn from(id: Id) -> Self {
        id.0
    }
}

impl Deref for Id {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.to_string().fmt(f)
    }
}

impl FromStr for Id {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl FromPyObject<'_> for Id {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let string: String = ob.extract()?;
        let uuid = Uuid::parse_str(&string).map_err(|e| {
            PyValueError::new_err(format!("Invalid UUID: {}", e))
        })?;
        Ok(Self(uuid))
    }
}

impl ToPyObject for Id {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_string().to_object(py)
    }
}

impl IntoPy<PyObject> for Id {
    fn into_py(self, py: Python) -> PyObject {
        self.to_object(py)
    }
}

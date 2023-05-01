use std::str::FromStr;
use std::fmt;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use heck::AsKebabCase;
use thiserror::Error;
use arrayvec::ArrayString;

use super::Name;

pub const MAX_LEN: usize = 64; // 64 characters should be enough; see https://github.com/rust-lang/crates.io/issues/696

/// Package ID.
/// It is used to uniquely identify a package, no two packages in the same registry can have the same ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Id(ArrayString<MAX_LEN>);

/// Errors that can occur when validating a package name.
#[derive(Error, Debug)]
pub enum Error {
    /// ID is not in lower kebab-case.
    #[error("package ID must be in lower kebab-case")]
    NotKebabCase,

    /// ID is longer than 64 characters.
    #[error("package ID must not be longer than 64 characters")]
    TooLong,

    /// ID is too short, or empty. IDs must be at least 3 characters long.
    #[error("package ID must have at least 3 characters")]
    TooShort,
}

mod python_exception {
    #![allow(missing_docs)]
    pyo3::create_exception!(merlon, Error, pyo3::exceptions::PyValueError);
}

impl Id {
    /// Creates a new ID from a package name.
    pub fn generate_for_package_name(name: &Name) -> Self {
        let mut s = name.as_kebab_case();
        if s.len() > 3 {
            s.truncate(64);
            s
        } else {
            const PREFIX: &'static str = "merlon-";
            s.truncate(MAX_LEN - PREFIX.len());
            format!("{}-{}", PREFIX, s)
        }.parse().expect("ID is valid")
    }
}

impl FromStr for Id {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Validate the ID.
        if s.len() < 3 {
            return Err(Error::TooShort);
        }
        if s != &format!("{}", AsKebabCase(&s)) {
            return Err(Error::NotKebabCase);
        }
        // Convert it
        let s = ArrayString::from(&s).map_err(|_| Error::TooLong)?;
        Ok(Self(s))
    }
}

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.to_string().fmt(f)
    }
}

impl FromPyObject<'_> for Id {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let s: String = ob.extract()?;
        Self::from_str(&s).map_err(|e| python_exception::Error::new_err(e.to_string()))
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

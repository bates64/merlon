use std::str::FromStr;
use std::fmt;
use heck::AsKebabCase;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use pyo3::prelude::*;

/// A validated package name.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Name(String);

/// Errors that can occur when validating a package name.
#[derive(Error, Debug)]
pub enum Error {
    /// Package name is empty.
    #[error("package name cannot be empty")]
    Empty,

    /// Package name contains a forward slash.
    #[error("package name cannot contain '/'")]
    ContainsSlash,

    /// Package name is multiple lines (contains a newline character).
    #[error("package name must be single line")]
    ContainsNewline,
}

mod python_exception {
    #![allow(missing_docs)]
    pyo3::create_exception!(merlon, NameError, pyo3::exceptions::PyValueError);
}

/// Package validation result type alias.
pub type Result<T> = std::result::Result<T, Error>;

/// Trait alias for TryInto<Name, Error = Error>
pub trait TryIntoName {
    /// Try to convert into a package name.
    fn try_into_name(self) -> Result<Name>;
}

impl<T: TryInto<Name, Error=Error>> TryIntoName for T {
    fn try_into_name(self) -> Result<Name> {
        self.try_into()
    }
}

impl TryIntoName for Name {
    fn try_into_name(self) -> Result<Name> {
        Ok(self)
    }
}

impl Name {
    /// Creates a new name from a string.
    pub fn new(name: String) -> Result<Self> {
        if name.is_empty() { 
            return Err(Error::Empty);
        }
        if name.contains('/') {
            return Err(Error::ContainsSlash);
        }
        if name.contains('\n') {
            return Err(Error::ContainsNewline);
        }
        Ok(Self(name))
    }

    /// Returns the name as kebab-case.
    pub fn as_kebab_case(&self) -> String {
        format!("{}", AsKebabCase(&self.0))
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Name {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        Self::new(s.to_owned())
    }
}

impl TryFrom<String> for Name {
    type Error = Error;
    fn try_from(value: String) -> Result<Self> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Name {
    type Error = Error;
    fn try_from(value: &str) -> Result<Self> {
        Self::new(value.to_owned())
    }
}

impl<'name> Into<&'name str> for &'name Name {
    fn into(self) -> &'name str {
        &self.0
    }
}

impl FromPyObject<'_> for Name {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let s: String = ob.extract()?;
        Self::new(s).map_err(|e| python_exception::NameError::new_err(e.to_string()))
    }
}

impl ToPyObject for Name {
    fn to_object(&self, py: Python) -> PyObject {
        self.0.to_object(py)
    }
}

impl IntoPy<PyObject> for Name {
    fn into_py(self, py: Python) -> PyObject {
        self.0.into_py(py)
    }
}

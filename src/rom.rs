//! ROM file handling

use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::fmt;
use sha1::{Sha1, Digest};
use anyhow::Result;
use pyo3::prelude::*;
use serde::{Serialize, Deserialize};

/// The file extensions that are supported to be opened as ROMs.
pub const EXTENSIONS: &[&str] = &["z64"]; // TODO: auto-convert n64 and v64 to z64

/// The SHA1 hash of the Paper Mario (US) ROM.
pub const PAPERMARIO_US_SHA1: &str = "3837f44cda784b466c9a2d99df70d77c322b97a0";

/// An N64 ROM file on disk.
#[derive(Debug, Serialize, Deserialize)]
#[pyclass(module = "merlon.rom")]
pub struct Rom {
    path: PathBuf,
}

#[pymethods]
impl Rom {
    /// Returns the path to the ROM file.
    #[getter]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Reads the ROM file into a [`Vec`] of bytes.
    pub fn read_bytes(&self) -> std::io::Result<Vec<u8>> {
        let mut file = self.file()?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
    
    /// Calculates the SHA1 hash of the ROM.
    pub fn sha1_string(&self) -> Result<String> {
        let mut bytes = self.read_bytes()?;
        let generic_arr = Sha1::digest(&mut bytes);
        let mut hex = String::new();
        for byte in generic_arr.iter() {
            hex.push_str(&format!("{:02x}", byte));
        }
        Ok(hex)
    }

    fn __str__(&self) -> String {
        format!("{}", self)
    }
}

impl Rom {
    /// Returns the ROM as a [`File`].
    pub fn file(&self) -> std::io::Result<File> {
        File::open(self.path())
    }
}

impl From<PathBuf> for Rom {
    fn from(path: PathBuf) -> Self {
        Self { path }
    }
}

impl fmt::Display for Rom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path().display())?;
        if let Ok(sha1) = self.sha1_string() {
            write!(f, " (SHA1: {})", sha1)?;
        }
        Ok(())
    }
}

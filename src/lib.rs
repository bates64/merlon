#![warn(missing_docs)] // Required for Python API doc generation

//! Merlon is a mod package manager for the Paper Mario (N64) decompilation.
//!
//! Merlon is also available as a Python library: https://pypi.org/project/merlon/

pub mod package;
pub mod emulator;
pub mod rom;

mod python;

use pyo3::prelude::*;
use super::package;

#[pymodule]
fn merlon(py: Python, merlon: &PyModule) -> PyResult<()> {
    // Mirror public Rust API.
    // Remember to also update `docs/api.rst`.
    merlon.add_function(wrap_pyfunction!(version, merlon)?)?;
    merlon.add_submodule({
        let package = PyModule::new(py, "package")?;
        package.add_class::<package::Package>()?;
        package.add_submodule({
            let manifest = PyModule::new(py, "manifest")?;
            manifest.add_class::<package::manifest::Manifest>()?;
            manifest.add_class::<package::manifest::Metadata>()?;
            manifest
        })?;
        package
    })?;
    Ok(())
}

/// Returns the current version of Merlon as a string.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

use pyo3::prelude::*;
use super::package;

#[pymodule]
fn merlon(py: Python, merlon: &PyModule) -> PyResult<()> {
    // Mirror public Rust API
    merlon.add_class::<package::Package>()?;
    merlon.add_submodule({
        let package = PyModule::new(py, "package")?;
        package.add_class::<package::Package>()?;
        package.add_class::<package::manifest::Manifest>()?;
        //package.add_class::<package::registry::Registry>()?;
        //package.add_class::<package::init::InitialisedPackage>()?;
        //package.add_class::<package::distribute::Distributable>()?;
        package
    })?;
    Ok(())
}

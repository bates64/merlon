use pyo3::prelude::*;
use super::*;

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
        package.add_submodule({
            let distribute = PyModule::new(py, "distribute")?;
            distribute.add_class::<package::distribute::Distributable>()?;
            distribute.add_class::<package::distribute::ExportOptions>()?;
            distribute.add_class::<package::distribute::ApplyOptions>()?;
            distribute.add_class::<package::distribute::OpenOptions>()?;
            distribute
        })?;
        package.add_submodule({
            let init = PyModule::new(py, "init")?;
            init.add_class::<package::init::InitialisedPackage>()?;
            init.add_class::<package::init::InitialiseOptions>()?;
            init.add_class::<package::init::BuildRomOptions>()?;
            init.add_class::<package::init::AddDependencyOptions>()?;
            init
        })?;
        package.add_submodule({
            let registry = PyModule::new(py, "registry")?;
            registry.add_class::<package::Registry>()?;
            registry
        })?;
        package
    })?;
    merlon.add_submodule({
        let emulator = PyModule::new(py, "emulator")?;
        emulator.add_function(wrap_pyfunction!(emulator::run_rom, emulator)?)?;
        emulator
    })?;
    merlon.add_submodule({
        let rom = PyModule::new(py, "rom")?;
        rom.add_class::<rom::Rom>()?;
        rom
    })?;
    Ok(())
}

/// Returns the current version of Merlon as a string.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

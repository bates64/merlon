use pyo3::prelude::*;

#[pymodule]
fn merlon(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello, m)?)?;
    Ok(())
}

#[pyfunction]
fn hello() -> &'static str {
    "Hello, world!"
}

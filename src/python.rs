use crate::prelude::*;
use pyo3::prelude::*;

#[pymodule]
fn sp3(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Epoch>()?;
    m.add_class::<TimeScale>()?;
    m.add_class::<Constellation>()?;
    m.add_class::<SV>()?;
    Ok(())
}

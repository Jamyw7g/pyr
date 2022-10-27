use std::ops::{Deref, DerefMut};

use pyo3::{
    types::PyBytes, FromPyObject, IntoPy, PyAny, PyObject, PyResult, PyTryFrom, ToPyObject,
};

#[derive(Debug, Clone)]
pub struct BoxedBytes {
    bytes: Box<[u8]>,
}

#[allow(dead_code)]
impl BoxedBytes {
    pub fn new(bytes: Box<[u8]>) -> Self {
        Self { bytes }
    }
}

impl Deref for BoxedBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl DerefMut for BoxedBytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytes
    }
}

impl From<&[u8]> for BoxedBytes {
    fn from(s: &[u8]) -> Self {
        Self { bytes: s.into() }
    }
}

impl<const N: usize> From<[u8; N]> for BoxedBytes {
    fn from(arr: [u8; N]) -> Self {
        Self { bytes: arr.into() }
    }
}

impl From<Vec<u8>> for BoxedBytes {
    fn from(vec: Vec<u8>) -> Self {
        Self { bytes: vec.into() }
    }
}

impl IntoPy<PyObject> for BoxedBytes {
    fn into_py(self, py: pyo3::Python<'_>) -> PyObject {
        <Self as ToPyObject>::to_object(&self, py)
    }
}

impl ToPyObject for BoxedBytes {
    fn to_object(&self, py: pyo3::Python<'_>) -> PyObject {
        PyBytes::new(py, &self.bytes).to_object(py)
    }
}

impl<'source> FromPyObject<'source> for BoxedBytes {
    fn extract(obj: &'source PyAny) -> PyResult<Self> {
        Ok(Self::from(
            <PyBytes as PyTryFrom>::try_from(obj)?.as_bytes(),
        ))
    }
}

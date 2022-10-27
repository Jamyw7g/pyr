use std::str::FromStr;

use pyo3::prelude::*;
use reqwest::{
    header::{HeaderMap as RHeader, HeaderName, HeaderValue, IntoIter},
    StatusCode,
};

use crate::types::BoxedBytes;

#[pyclass]
#[derive(Debug, Clone)]
pub struct Response {
    pub(crate) url: String,
    pub(crate) content: BoxedBytes,
    pub(crate) status: StatusCode,
    pub(crate) header: Header,
}

#[pymethods]
impl Response {
    #[getter(url)]
    fn get_url(&self) -> &str {
        &self.url
    }

    #[getter(content)]
    fn get_content(&self) -> &[u8] {
        &self.content
    }

    fn headers(&self) -> Header {
        self.header.clone()
    }

    fn ok(&self) -> bool {
        self.status.is_success()
    }
}

#[pyclass]
#[derive(Debug, Clone)]
pub struct Header {
    header: RHeader,
}

#[pymethods]
impl Header {
    fn __getitem__(slf: PyRef<Self>, key: &str) -> Option<String> {
        let key = HeaderName::from_str(key).ok()?;
        slf.header
            .get(&key)
            .map(|val| val.to_str().unwrap_or_default().to_owned())
    }

    fn __iter__(slf: PyRef<Self>) -> HeaderIter {
        HeaderIter {
            inner: slf.header.clone().into_iter(),
        }
    }

    fn __str__(slf: PyRef<Self>) -> String {
        format!("{:?}", slf.header)
    }
}

impl From<&RHeader> for Header {
    fn from(header: &RHeader) -> Self {
        Self {
            header: header.clone(),
        }
    }
}

#[pyclass]
struct HeaderIter {
    inner: IntoIter<HeaderValue>,
}

#[pymethods]
impl HeaderIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> Option<(Option<String>, Option<String>)> {
        slf.inner.next().map(|(k, v)| {
            (
                k.map(|k| k.as_str().to_owned()),
                v.to_str().map(|v| v.to_owned()).ok(),
            )
        })
    }
}

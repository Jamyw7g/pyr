use pyo3::{
    exceptions::{PyException, PyValueError},
    PyErr,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    StdIoErr(#[from] std::io::Error),
    #[error(transparent)]
    ReqErr(#[from] reqwest::Error),
    #[error(transparent)]
    AnyError(#[from] anyhow::Error),
    #[error(transparent)]
    PyErr(#[from] PyErr),
    #[error("None")]
    None,
}

impl From<Error> for PyErr {
    fn from(e: Error) -> Self {
        match e {
            Error::StdIoErr(e) => e.into(),
            Error::ReqErr(e) => PyException::new_err(e.to_string()),
            Error::AnyError(e) => PyException::new_err(e.to_string()),
            Error::PyErr(e) => e,
            Error::None => PyValueError::new_err("None"),
        }
    }
}

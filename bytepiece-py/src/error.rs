use pyo3::{exceptions::PyValueError, prelude::*};

pub type Result<T> = std::result::Result<T, Error>;

pub struct Error(bytepiece::Error);

impl From<Error> for PyErr {
    fn from(value: Error) -> Self {
        PyValueError::new_err(value.0.to_string())
    }
}

impl From<bytepiece::Error> for Error {
    fn from(value: bytepiece::Error) -> Self {
        Self(value)
    }
}

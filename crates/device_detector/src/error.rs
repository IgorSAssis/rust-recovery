use thiserror::Error;

#[derive(Debug, Error)]
pub enum DeviceDetectorError {
    #[error("I/O error reading '{path}': {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid value for field '{field}' (got {raw_value:?})")]
    Parse {
        field: &'static str,
        raw_value: String,
    },
}

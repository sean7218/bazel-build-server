pub type Error = BSPError;
pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum BSPError {
    Custom(String),
    TargetNotFound(String),
    JsonError(serde_json::Error),
    IoError(std::io::Error)
}

impl std::fmt::Display for BSPError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BSPError::Custom(e) => {
                write!(f, "BSPError::Custom -> Reason: {}\n", e)
            }
            BSPError::TargetNotFound(e) => {
                write!(f, "BSPError::TargetNotFound -> Reason: {}\n", e)
            }
            BSPError::JsonError(e) => {
                write!(f, "BSPError::JsonError -> Reason: {}\n", e)
            }
            BSPError::IoError(e) => {
                write!(f, "BSPError::IoError -> Reason: {}\n", e)
            }
        }
    }
}

impl std::error::Error for BSPError {}

impl From<&str> for BSPError {
    fn from(err: &str) -> Self {
        BSPError::Custom(err.to_string())
    }
}

impl From<serde_json::Error> for BSPError {
    fn from(err: serde_json::Error) -> Self {
        BSPError::JsonError(err)
    }
}

impl From<std::io::Error> for BSPError {
    fn from(err: std::io::Error) -> Self {
        BSPError::IoError(err)
    }
}

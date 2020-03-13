#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("NULL pointer encountered")]
    NullPointer,
    #[error("String contains invalid Utf8 character: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("Message: {0}")]
    Message(crate::Message),
    #[error("I/O: {0}")]
    IO(#[from] std::io::Error),
    #[error("Invalid path name")]
    InvalidPath,
    #[error("Invalid type")]
    InvalidType,
    #[error("Invalid constant")]
    InvalidConst,
}

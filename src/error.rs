#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("NULL pointer encountered")]
    NullPointer,
    #[error("String contains invalid Utf8 character: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("Message: {0}")]
    Message(crate::Message),
    #[error("Invalid path name")]
    InvalidPath,
}

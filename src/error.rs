/// An enumeration of all possible errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// NULL pointer encountered, this could be due to an invalid use of LLVM API or an actual
    /// error
    #[error("NULL pointer encountered")]
    NullPointer,

    /// UTF conversion failed
    #[error("String contains invalid Utf8 character: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    /// LLVM message
    #[error("Message: {0}")]
    Message(crate::Message),

    /// IO error
    #[error("I/O: {0}")]
    IO(#[from] std::io::Error),

    /// Invalid path name
    #[error("Invalid path name")]
    InvalidPath,

    /// Invalid LLVM type was encountered
    #[error("Invalid type")]
    InvalidType,

    /// Invalid LLVM constant
    #[error("Value is not a constant")]
    InvalidConst,

    /// Invalid LLVM basic block
    #[error("Value is not a basic block")]
    InvalidBasicBlock,

    /// Invalid LLVM function
    #[error("Invalid function")]
    InvalidFunction,

    /// Module is already owned by another ExecutionEngine
    #[error("Module is already owned by another execution engine")]
    ModuleIsAlreadyOwned,

    /// Mutex guard poison error
    #[error("Poison error: {0}")]
    Poison(#[from] std::sync::PoisonError<std::sync::MutexGuard<'static, ()>>),
}

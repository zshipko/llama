use crate::*;

/// Wraps LLVM messages, these are strings that should be freed using LLVMDisposeMessage
pub struct Message(pub(crate) *mut c_char);
impl Message {
    pub(crate) fn from_raw(c: *mut c_char) -> Message {
        Message(c)
    }

    /// Message length
    pub fn len(&self) -> usize {
        if self.0.is_null() {
            return 0;
        }

        unsafe { strlen(self.0) }
    }

    /// Returns true when the message is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl AsRef<str> for Message {
    fn as_ref(&self) -> &str {
        if self.0.is_null() {
            return "<NULL>";
        }

        unsafe {
            let st = std::slice::from_raw_parts(self.0 as *const u8, self.len());
            std::str::from_utf8_unchecked(st)
        }
    }
}

impl From<Message> for String {
    fn from(m: Message) -> String {
        m.as_ref().into()
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { llvm::core::LLVMDisposeMessage(self.0) }
        }
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self.as_ref())
    }
}

impl std::fmt::Debug for Message {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(fmt, "{}", self)
    }
}

#[derive(Debug, Clone)]
pub struct NotLoadedError;

impl std::fmt::Display for NotLoadedError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "The chunk being modified has not been loaded")
    }
}
impl std::error::Error for NotLoadedError {}

/// Errors involving `BlockOffset`s.
#[derive(Debug)]
pub enum OffsetError {
    /// The offset failed because the resulting position would be in
    /// a different chunk.
    DifferentChunk,
    /// The offset failed because the resulting position's y-level would
    /// be outside of `0..=512`.
    OutOfBounds,
}

impl std::fmt::Display for OffsetError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            OffsetError::DifferentChunk => write!(f, "tried to offset into different chunk"),
            OffsetError::OutOfBounds => write!(f, "tried to offset outside y bounds"),
        }
    }
}

impl std::error::Error for OffsetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

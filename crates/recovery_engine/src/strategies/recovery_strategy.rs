use crate::error::EngineError;
use crate::types::{ExtractedFile, ReadSeek};

pub trait RecoveryStrategy {
    fn recover(&self, source: &mut dyn ReadSeek) -> Result<Vec<ExtractedFile>, EngineError>;
}

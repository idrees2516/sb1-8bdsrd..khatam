#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Field arithmetic error: {0}")]
    FieldError(#[from] crate::field::FieldError),
    
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Protocol verification failed: {0}")]
    VerificationError(String),
    
    #[error("Encoding error: {0}")]
    EncodingError(String),
    
    #[error("Decoding error: {0}")]
    DecodingError(String),
    
    #[error("Proof generation failed: {0}")]
    ProofError(String),
    
    #[error("System error: {0}")]
    SystemError(String),
}
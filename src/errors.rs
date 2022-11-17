use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Failure reading/writing from file")]
    File(#[from] std::io::Error),

    #[error("Failure serializing/deserializing entries")]
    SerializeSpanned(#[from] ron::error::SpannedError),

    #[error("Failure serializing/deserializing entries")]
    Serialize(#[from] ron::error::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

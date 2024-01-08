use thiserror::Error;


#[derive(Error, Debug)]
pub enum StructError {
    #[error("Incorrect number of args: expect {0}")]
    IncorrectArgCount(u8),

    #[error("Incorrect number of args: expect {0} or more")]
    ExpectMoreArg(u8),
}

pub type Result<T> = std::result::Result<T, StructError>;

mod evenger;
mod error;
mod srcdev;
mod destdev;

pub use evenger::Evenger;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

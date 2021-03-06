
mod evenger;
mod error;
mod srcdev;
mod destdev;
mod rule;

pub use evenger::Evenger;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;
pub type DeviceId = std::rc::Rc<String>;

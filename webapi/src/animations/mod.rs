mod logic;
mod service;
mod storage;

pub use logic::{Logic, LogicError};
pub use service::service;
use storage::Storage;

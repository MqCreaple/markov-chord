use std::result;

pub type Error = String;
pub type Result<T> = result::Result<T, Error>;

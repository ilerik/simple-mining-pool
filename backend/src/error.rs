//! Crate level errors

use std::{fmt, error};

use jwt;
use json;

#[derive(Debug, PartialEq, Clone)]
/// Errors 
pub enum Error {
	ArbitraryFailure(String),
}

impl From<jwt::Error> for Error {
	fn from(err: jwt::Error) -> Self {
		Error::ArbitraryFailure(format!("{:?}", err))
	}
}

impl From<json::Error> for Error {
	fn from(err: json::Error) -> Self {
		Error::ArbitraryFailure(format!("{:?}", err))
	}
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use self::Error::*;
		let msg = match *self {			
			ArbitraryFailure(ref err) => format!("Core error: {}.", err),			
		};

		f.write_fmt(format_args!("Core error ({})", msg))
	}
}

impl error::Error for Error {
	fn description(&self) -> &str {
		"Core error"
	}
}
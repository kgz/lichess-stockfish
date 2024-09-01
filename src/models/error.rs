use std::fmt;

#[derive(Debug)]
pub enum Error {
	BasicError(String),
}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Error::BasicError(e) => write!(f, "{}", e),
		}
	}

	
}


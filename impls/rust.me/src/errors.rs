#[derive(Debug)]
pub enum MalErr {
    // read
    ReadErr(String),
    // env
    SymbolNotFound(String),
    // eval
    InvalidLet(String),
    Generic(String),
}

impl std::fmt::Display for MalErr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MalErr::ReadErr(message) => write!(f, "Read error: {}", message),
            MalErr::SymbolNotFound(symbol) => write!(f, "{} not found", symbol),
            MalErr::InvalidLet(message) => write!(f, "Invalid let* construction: {}", message),
            MalErr::Generic(message) => write!(f, "Error: {}", message),
        }
    }
}

impl std::error::Error for MalErr {}

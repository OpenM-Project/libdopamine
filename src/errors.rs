use std::fmt;

#[derive(Debug)]
///
/// # Error type enumerator
/// See `libdopamine::errors::DopamineError` for usage information
pub enum ErrorType {
    ReadWriteError,
    ProtectBypassError,
    QueryError,
    ProcessClosedError
}

#[derive(Debug)]
///
/// # Error struct for libdopamine
/// 
/// ## Example usage
/// ```rust
/// use std::process::exit;
/// use libdopamine;
/// 
/// match libdopamine::module::inject_module(process, module, &mut data, false) {
///     Ok(_) => {}, Err(err) => {
///         if let libdopamine::errors::ErrorType::ReadWriteError = err.error_type {
///             eprintln!("R/W error while injecting module");
///             exit(1);
///         } else {
///             eprintln!("Uncaught error while injecting module: {}", err);
///             exit(1);
///         }
///     }
/// };
/// ```
/// 
pub struct DopamineError {
    pub message: String,
    pub error_type: ErrorType,
}

impl DopamineError {
    pub fn new(message: &str, error_type: ErrorType) -> Self {
        DopamineError {
            message: message.to_string(),
            error_type,
        }
    }
}

impl fmt::Display for DopamineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DopamineError ({:?}): {}", self.error_type, self.message)
    }
}

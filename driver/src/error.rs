use obfstr::obfstr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InterfaceError {
    #[error("this platform is not supported")]
    PlatformUnsupported,

    #[error("process unknown")]
    ProcessUnknown,

    #[error("process is ubiquitous")]
    ProcessUbiquitous,

    #[error("failed to open process {process_id}: {message}")]
    ProcessOpenFailed { process_id: u32, message: String },

    #[error("failed to access memory")]
    MemoryAccessFailed,

    #[error("{operation} failed: {message}")]
    SystemCallFailed {
        operation: &'static str,
        message: String,
    },
}

pub type IResult<T> = std::result::Result<T, InterfaceError>;

impl InterfaceError {
    pub fn detailed_message(&self) -> Option<String> {
        Some(match self {
            InterfaceError::ProcessUnknown => [
                obfstr!("Could not find CS2 process."),
                obfstr!("Please start CS2 prior to executing this application!"),
            ]
            .join("\n"),
            InterfaceError::ProcessUbiquitous => [
                obfstr!("Found more than one CS2 process."),
                obfstr!("Please close all but one instance of CS2 and try again."),
            ]
            .join("\n"),
            InterfaceError::ProcessOpenFailed { .. } => [
                obfstr!("Could not open the CS2 process for reading."),
                obfstr!("If CS2 runs as administrator, run this application as administrator too."),
            ]
            .join("\n"),
            InterfaceError::PlatformUnsupported => {
                obfstr!("Reading CS2 memory is only supported on Windows.").to_string()
            }
            _ => return None,
        })
    }
}

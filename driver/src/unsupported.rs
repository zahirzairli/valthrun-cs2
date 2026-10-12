use crate::{
    IResult,
    InterfaceError,
    KeyboardState,
    MouseState,
    ProcessId,
    ProcessInfo,
    ProcessModuleInfo,
};

pub struct Platform;

impl Platform {
    pub fn create() -> IResult<Self> {
        Err(InterfaceError::PlatformUnsupported)
    }

    pub fn list_processes(&self) -> IResult<Vec<ProcessInfo>> {
        Err(InterfaceError::PlatformUnsupported)
    }

    pub fn list_modules(&self, _process_id: ProcessId) -> IResult<Vec<ProcessModuleInfo>> {
        Err(InterfaceError::PlatformUnsupported)
    }

    pub fn read(&self, _process_id: ProcessId, _address: u64, _buffer: &mut [u8]) -> IResult<()> {
        Err(InterfaceError::PlatformUnsupported)
    }

    pub fn send_keyboard_state(&self, _states: &[KeyboardState]) -> IResult<()> {
        Err(InterfaceError::PlatformUnsupported)
    }

    pub fn send_mouse_state(&self, _states: &[MouseState]) -> IResult<()> {
        Err(InterfaceError::PlatformUnsupported)
    }
}

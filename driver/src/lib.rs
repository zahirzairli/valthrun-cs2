//! Memory access and input for the CS2 controller.
//!
//! Everything runs inside the calling process using standard Windows APIs:
//! processes and modules are listed with a ToolHelp snapshot, memory is read with
//! `ReadProcessMemory` and input is sent with `SendInput`.
//! Nothing is written to the target process and nothing is sent over the network.

use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};

mod error;
mod types;

#[cfg(windows)]
#[path = "windows.rs"]
mod platform;

#[cfg(not(windows))]
#[path = "unsupported.rs"]
mod platform;

pub use error::*;
pub use types::*;

pub struct DriverInterface {
    platform: platform::Platform,
    read_calls: AtomicUsize,
}

impl DriverInterface {
    pub fn create() -> IResult<Self> {
        Ok(Self {
            platform: platform::Platform::create()?,
            read_calls: AtomicUsize::new(0),
        })
    }

    #[must_use]
    pub fn total_read_calls(&self) -> usize {
        self.read_calls.load(Ordering::Relaxed)
    }

    pub fn list_processes(&self) -> IResult<Vec<ProcessInfo>> {
        self.platform.list_processes()
    }

    pub fn list_modules(&self, process_id: ProcessId) -> IResult<Vec<ProcessModuleInfo>> {
        self.platform.list_modules(process_id)
    }

    pub fn read<T: Copy>(&self, process_id: ProcessId, address: u64) -> IResult<T> {
        let mut result = std::mem::MaybeUninit::<T>::zeroed();
        let buffer = unsafe {
            std::slice::from_raw_parts_mut(result.as_mut_ptr() as *mut u8, size_of::<T>())
        };

        self.read_bytes(process_id, address, buffer)?;
        Ok(unsafe { result.assume_init() })
    }

    pub fn read_slice<T: Copy>(
        &self,
        process_id: ProcessId,
        address: u64,
        buffer: &mut [T],
    ) -> IResult<()> {
        let buffer = unsafe {
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut u8, size_of_val(buffer))
        };

        self.read_bytes(process_id, address, buffer)
    }

    fn read_bytes(&self, process_id: ProcessId, address: u64, buffer: &mut [u8]) -> IResult<()> {
        self.read_calls.fetch_add(1, Ordering::Relaxed);
        if buffer.is_empty() {
            return Ok(());
        }

        self.platform.read(process_id, address, buffer)
    }

    pub fn send_keyboard_state(&self, states: &[KeyboardState]) -> IResult<()> {
        self.platform.send_keyboard_state(states)
    }

    pub fn send_mouse_state(&self, states: &[MouseState]) -> IResult<()> {
        self.platform.send_mouse_state(states)
    }
}

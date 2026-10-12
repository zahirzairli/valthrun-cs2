use std::{
    collections::HashMap,
    ffi::c_void,
    sync::Mutex,
};

use windows::Win32::{
    Foundation::{
        CloseHandle,
        ERROR_BAD_LENGTH,
        ERROR_INVALID_PARAMETER,
        HANDLE,
    },
    System::{
        Diagnostics::{
            Debug::ReadProcessMemory,
            ToolHelp::{
                CreateToolhelp32Snapshot,
                Module32FirstW,
                Module32NextW,
                Process32FirstW,
                Process32NextW,
                MODULEENTRY32W,
                PROCESSENTRY32W,
                TH32CS_SNAPMODULE,
                TH32CS_SNAPMODULE32,
                TH32CS_SNAPPROCESS,
            },
        },
        Threading::{
            OpenProcess,
            PROCESS_QUERY_LIMITED_INFORMATION,
            PROCESS_VM_READ,
        },
    },
    UI::Input::KeyboardAndMouse::{
        SendInput,
        INPUT,
        INPUT_KEYBOARD,
        INPUT_MOUSE,
        KEYEVENTF_KEYUP,
        KEYEVENTF_SCANCODE,
        MOUSEEVENTF_HWHEEL,
        MOUSEEVENTF_LEFTDOWN,
        MOUSEEVENTF_LEFTUP,
        MOUSEEVENTF_MIDDLEDOWN,
        MOUSEEVENTF_MIDDLEUP,
        MOUSEEVENTF_MOVE,
        MOUSEEVENTF_RIGHTDOWN,
        MOUSEEVENTF_RIGHTUP,
        MOUSEEVENTF_WHEEL,
        MOUSEEVENTF_XDOWN,
        MOUSEEVENTF_XUP,
        MOUSE_EVENT_FLAGS,
    },
};

use crate::{
    IResult,
    InterfaceError,
    KeyboardState,
    MouseState,
    ProcessId,
    ProcessInfo,
    ProcessModuleInfo,
};

const WHEEL_DELTA: u32 = 120;
const XBUTTON1: u32 = 0x0001;
const XBUTTON2: u32 = 0x0002;

/// Closes the wrapped handle when dropped.
struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}

fn system_error(operation: &'static str, error: windows::core::Error) -> InterfaceError {
    InterfaceError::SystemCallFailed {
        operation,
        message: error.message(),
    }
}

fn utf16_to_string(value: &[u16]) -> String {
    let length = value.iter().position(|c| *c == 0).unwrap_or(value.len());
    String::from_utf16_lossy(&value[..length])
}

pub struct Platform {
    /// Process handles opened for reading, stored as raw values so the platform stays `Send + Sync`.
    /// They are kept open until the interface is dropped.
    process_handles: Mutex<HashMap<ProcessId, usize>>,
}

impl Platform {
    pub fn create() -> IResult<Self> {
        Ok(Self {
            process_handles: Default::default(),
        })
    }

    fn process_handle(&self, process_id: ProcessId) -> IResult<HANDLE> {
        let mut handles = self.process_handles.lock().unwrap();
        if let Some(handle) = handles.get(&process_id) {
            return Ok(HANDLE(*handle as *mut c_void));
        }

        let handle = unsafe {
            OpenProcess(
                PROCESS_VM_READ | PROCESS_QUERY_LIMITED_INFORMATION,
                false,
                process_id,
            )
        }
        .map_err(|err| InterfaceError::ProcessOpenFailed {
            process_id,
            message: err.message(),
        })?;

        handles.insert(process_id, handle.0 as usize);
        Ok(handle)
    }

    pub fn list_processes(&self) -> IResult<Vec<ProcessInfo>> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
            .map(OwnedHandle)
            .map_err(|err| system_error("CreateToolhelp32Snapshot", err))?;

        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        let mut processes = Vec::new();
        let mut next = unsafe { Process32FirstW(snapshot.0, &mut entry) };
        while next.is_ok() {
            processes.push(ProcessInfo {
                process_id: entry.th32ProcessID,
                image_name: utf16_to_string(&entry.szExeFile),
            });

            next = unsafe { Process32NextW(snapshot.0, &mut entry) };
        }

        Ok(processes)
    }

    pub fn list_modules(&self, process_id: ProcessId) -> IResult<Vec<ProcessModuleInfo>> {
        /* The snapshot may fail with ERROR_BAD_LENGTH while the process loads or unloads modules. */
        let mut attempt = 0;
        let snapshot = loop {
            match unsafe {
                CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, process_id)
            } {
                Ok(handle) => break OwnedHandle(handle),
                Err(err) if err.code() == ERROR_BAD_LENGTH.to_hresult() && attempt < 5 => {
                    attempt += 1;
                }
                Err(err) if err.code() == ERROR_INVALID_PARAMETER.to_hresult() => {
                    return Err(InterfaceError::ProcessUnknown);
                }
                Err(err) => return Err(system_error("CreateToolhelp32Snapshot", err)),
            }
        };

        let mut entry = MODULEENTRY32W {
            dwSize: size_of::<MODULEENTRY32W>() as u32,
            ..Default::default()
        };

        let mut modules = Vec::new();
        let mut next = unsafe { Module32FirstW(snapshot.0, &mut entry) };
        while next.is_ok() {
            modules.push(ProcessModuleInfo {
                name: utf16_to_string(&entry.szModule),
                base_address: entry.modBaseAddr as u64,
                module_size: entry.modBaseSize as u64,
            });

            next = unsafe { Module32NextW(snapshot.0, &mut entry) };
        }

        Ok(modules)
    }

    pub fn read(&self, process_id: ProcessId, address: u64, buffer: &mut [u8]) -> IResult<()> {
        let handle = self.process_handle(process_id)?;

        let mut bytes_read = 0usize;
        let result = unsafe {
            ReadProcessMemory(
                handle,
                address as *const c_void,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len(),
                Some(&mut bytes_read),
            )
        };

        if result.is_err() || bytes_read != buffer.len() {
            log::trace!(
                "Memory read failed at {:X} (len {:X}, read {:X})",
                address,
                buffer.len(),
                bytes_read
            );
            return Err(InterfaceError::MemoryAccessFailed);
        }

        Ok(())
    }

    pub fn send_keyboard_state(&self, states: &[KeyboardState]) -> IResult<()> {
        let inputs = states
            .iter()
            .map(|state| {
                let mut input = INPUT {
                    r#type: INPUT_KEYBOARD,
                    ..Default::default()
                };

                let ki = unsafe { &mut input.Anonymous.ki };
                ki.wScan = state.scan_code;
                ki.dwFlags = KEYEVENTF_SCANCODE;
                if !state.down {
                    ki.dwFlags |= KEYEVENTF_KEYUP;
                }

                input
            })
            .collect::<Vec<_>>();

        send_inputs(&inputs)
    }

    pub fn send_mouse_state(&self, states: &[MouseState]) -> IResult<()> {
        let mut inputs = Vec::with_capacity(states.len());
        for state in states {
            let flags = mouse_flags(state);
            if flags.0 != 0 {
                inputs.push(mouse_input(flags, 0, state));
            }

            /* X1 and X2 share the XDOWN / XUP flags, so each needs its own input. */
            for (button, mouse_data) in [(3, XBUTTON1), (4, XBUTTON2)] {
                if let Some(down) = state.buttons[button] {
                    let flags = if down { MOUSEEVENTF_XDOWN } else { MOUSEEVENTF_XUP };
                    inputs.push(mouse_input(flags, mouse_data, &MouseState::default()));
                }
            }
        }

        send_inputs(&inputs)
    }
}

impl Drop for Platform {
    fn drop(&mut self) {
        let handles = self.process_handles.get_mut().unwrap();
        for (_, handle) in handles.drain() {
            drop(OwnedHandle(HANDLE(handle as *mut c_void)));
        }
    }
}

fn mouse_flags(state: &MouseState) -> MOUSE_EVENT_FLAGS {
    let mut flags = MOUSE_EVENT_FLAGS(0);
    let buttons = [
        (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
    ];
    for (index, (down_flag, up_flag)) in buttons.into_iter().enumerate() {
        if let Some(down) = state.buttons[index] {
            flags |= if down { down_flag } else { up_flag };
        }
    }

    if state.wheel {
        flags |= MOUSEEVENTF_WHEEL;
    } else if state.hwheel {
        flags |= MOUSEEVENTF_HWHEEL;
    }

    if state.last_x != 0 || state.last_y != 0 {
        flags |= MOUSEEVENTF_MOVE;
    }

    flags
}

fn mouse_input(flags: MOUSE_EVENT_FLAGS, mouse_data: u32, state: &MouseState) -> INPUT {
    let mut input = INPUT {
        r#type: INPUT_MOUSE,
        ..Default::default()
    };

    let mi = unsafe { &mut input.Anonymous.mi };
    mi.dwFlags = flags;
    mi.dx = state.last_x;
    mi.dy = state.last_y;
    mi.mouseData = if state.wheel || state.hwheel {
        WHEEL_DELTA
    } else {
        mouse_data
    };

    input
}

fn send_inputs(inputs: &[INPUT]) -> IResult<()> {
    if inputs.is_empty() {
        return Ok(());
    }

    let sent = unsafe { SendInput(inputs, size_of::<INPUT>() as i32) };
    if sent as usize != inputs.len() {
        return Err(system_error("SendInput", windows::core::Error::from_win32()));
    }

    Ok(())
}

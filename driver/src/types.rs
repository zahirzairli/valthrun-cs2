pub type ProcessId = u32;

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub process_id: ProcessId,
    pub image_name: String,
}

#[derive(Debug, Clone)]
pub struct ProcessModuleInfo {
    pub name: String,
    pub base_address: u64,
    pub module_size: u64,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct KeyboardState {
    pub scan_code: u16,
    pub down: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MouseState {
    /// Left, right, middle, X1 and X2.
    /// `None` leaves the button unchanged.
    pub buttons: [Option<bool>; 0x05],
    pub hwheel: bool,
    pub wheel: bool,

    pub last_x: i32,
    pub last_y: i32,
}

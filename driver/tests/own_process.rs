//! Runs the driver against the test process itself.
#![cfg(windows)]

use driver::DriverInterface;

static KNOWN_VALUE: [u32; 4] = [0xDEAD_BEEF, 0x1234_5678, 0, u32::MAX];

#[test]
fn finds_own_process_and_modules() {
    let interface = DriverInterface::create().unwrap();
    let process_id = std::process::id();

    let process = interface
        .list_processes()
        .unwrap()
        .into_iter()
        .find(|process| process.process_id == process_id)
        .expect("own process missing");

    let exe_name = std::env::current_exe().unwrap();
    let exe_name = exe_name.file_name().unwrap().to_str().unwrap();
    assert!(process.image_name.eq_ignore_ascii_case(exe_name));

    let modules = interface.list_modules(process_id).unwrap();
    let ntdll = modules
        .iter()
        .find(|module| module.name.eq_ignore_ascii_case("ntdll.dll"))
        .expect("ntdll.dll missing");
    assert!(ntdll.base_address != 0 && ntdll.module_size != 0);

    /* Every PE image starts with the "MZ" DOS header. */
    let header = interface.read::<[u8; 2]>(process_id, ntdll.base_address).unwrap();
    assert_eq!(&header, b"MZ");
}

#[test]
fn reads_own_memory() {
    let interface = DriverInterface::create().unwrap();
    let process_id = std::process::id();
    let address = KNOWN_VALUE.as_ptr() as u64;

    assert_eq!(interface.read::<u32>(process_id, address).unwrap(), 0xDEAD_BEEF);

    let mut buffer = [0u32; 4];
    interface
        .read_slice(process_id, address, &mut buffer)
        .unwrap();
    assert_eq!(buffer, KNOWN_VALUE);
    assert_eq!(interface.total_read_calls(), 2);
}

#[test]
fn reading_unmapped_memory_fails() {
    let interface = DriverInterface::create().unwrap();
    assert!(interface.read::<u64>(std::process::id(), 0).is_err());
}

#[test]
fn unknown_process_fails() {
    let interface = DriverInterface::create().unwrap();
    /* Process ids are multiples of four, so this one never exists. */
    assert!(interface.read::<u8>(3, 0x1000).is_err());
    assert!(interface.list_modules(3).is_err());
}

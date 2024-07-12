use crate::errors::*;
use std::{mem, str};
use windows::Win32::{
    Foundation::*,
    System::{Diagnostics::Debug::*, Memory::*, ProcessStatus::*, Threading::*},
};

const HMODULE_SIZE: usize = mem::size_of::<HMODULE>();

/// # `libdopamine::module::wait_for_module`
/// Blocking function to wait for a module to load.
/// 
/// ## Arguments
/// - **`process: HANDLE` ->** Process handle.
/// - **`module_name: &str` ->** Name of the module file.
/// 
/// ## Return values
/// **If `Ok` ->** `(module_handle: HMODULE, module_path: String)`
/// 
/// **If `Err` ->** `libdopamine::errors::DopamineError`
pub fn wait_for_module(process: HANDLE, module_name: &str) -> Result<(HMODULE, String), DopamineError> {
    let mut count: u32 = 128;
    loop {
        let mut cb_needed: u32 = 0;
        let mut modulelist: Vec<HMODULE>;
        loop {
            modulelist = vec![HMODULE::default(); count as usize * HMODULE_SIZE];
            match unsafe { EnumProcessModulesEx(
                process,
                modulelist.as_mut_ptr(),
                modulelist.len() as u32,
                &mut cb_needed,
                LIST_MODULES_ALL,
            ) } {
                Err(_) => {
                    let mut status: u32 = 0;
                    match unsafe { GetExitCodeProcess(process, &mut status as *mut u32) } {
                        Ok(_) => { if status != STILL_ACTIVE.0 as u32 {
                            let error_msg = format!("process is no longer active, exit code {}", status);
                            return Err(DopamineError::new(&error_msg, ErrorType::ProcessClosedError));
                        } }, _ => {}
                    }; continue;
                }, _ => {}
            }
            if cb_needed < modulelist.len() as u32 {
                modulelist.truncate(cb_needed as usize / HMODULE_SIZE); break
            };
            count *= 2;
        }
        for module in &modulelist {
            let mut mod_name: [u8; 1024] = [0 as u8; 1024];
            match unsafe { GetModuleFileNameExA(process, *module, &mut mod_name) } {
                0 => continue,
                _ => {}
            }
            let mod_name_str: &str = std::str::from_utf8(&mod_name[..]).unwrap().trim_end_matches("\0");
            if mod_name_str.split("\\").last().unwrap() == module_name {
                return Ok((*module, mod_name_str.to_string()));
            }
        }
    }
}

/// # `libdopamine::module::dump_module`
/// Blocking function to dump module from process memory.
/// 
/// ## Arguments
/// - **`process: HANDLE` ->** Process handle.
/// - **`module: HMODULE` ->** Module handle.
/// 
/// ## Return values
/// **If `Ok` ->** `(length: u32, data: Vec<u8>)`
/// 
/// **If `Err` ->** `libdopamine::errors::DopamineError`
pub fn dump_module(process: HANDLE, module: HMODULE) -> Result<(u32, Vec<u8>), DopamineError> {
    let mut module_info: MODULEINFO = MODULEINFO::default();
    match unsafe { GetModuleInformation(
        process,
        module,
        &mut module_info as *mut _,
        std::mem::size_of::<MODULEINFO>() as u32,
    ) } {
        Err(_) => {
            let error = unsafe { GetLastError() };
            let error_msg = format!("module info query fail, GetModuleInformation return {}", error.0);
            return Err(DopamineError::new(&error_msg, ErrorType::QueryError));
        }, _ => {}
    }
    let mut dump: Vec<u8> = vec![0; module_info.SizeOfImage as usize];
    let mut bytes_read: usize = 0;
    match unsafe { ReadProcessMemory(
        process,
        module_info.lpBaseOfDll,
        dump.as_mut_ptr() as *mut _,
        module_info.SizeOfImage as usize,
        Some(&mut bytes_read as *mut _),
    ) } {
        Err(_) => {
            let error = unsafe { GetLastError() };
            let error_msg = format!("read error, ReadProcessMemory return {}", error.0);
            return Err(DopamineError::new(&error_msg, ErrorType::ReadWriteError));
        }, _ => Ok((bytes_read as u32, dump))
    }
}

/// # `libdopamine::module::inject_module`
/// Blocking function to inject module inject process memory.
/// 
/// ## Arguments
/// - **`process: HANDLE` ->** Process handle.
/// - **`module: HMODULE` ->** Module handle.
/// - **`data: &mut Vec<u8>` ->** Bytes for injected module.
/// - **`ignore_security_fix: bool` ->** Don't write back
/// the old security status of the module after writing
/// security as PAGE_EXECUTE_READWRITE.
/// 
/// ## Return values
/// **If `Ok` ->** `()`
/// 
/// **If `Err` ->** `libdopamine::errors::DopamineError`
pub fn inject_module(process: HANDLE, module: HMODULE, data: &mut Vec<u8>, ignore_security_fix: bool) -> Result<(), DopamineError> {
    let data_ptr: *mut u8 = data.as_mut_ptr();
    let mut old_security = PAGE_PROTECTION_FLAGS::default();
    match unsafe { VirtualProtectEx(
            process,
            module.0,
            data.len(),
            PAGE_EXECUTE_READWRITE,
            &mut old_security as *mut _,
        )
    } {
        Err(_) => {
            let error = unsafe { GetLastError() };
            let error_msg = format!("security error, VirtualProtectEx return {}", error.0);
            return Err(DopamineError::new(&error_msg, ErrorType::ProtectBypassError));
        }, _ => {}
    };
    match unsafe { WriteProcessMemory(
            process,
            module.0,
            data_ptr as *mut _,
            data.len(),
            None,
        )
    } {
        Err(_) => {
            let error = unsafe { GetLastError() };
            let error_msg = format!("write error, WriteProcessMemory return {}", error.0);
            return Err(DopamineError::new(&error_msg, ErrorType::ReadWriteError));
        }, _ => {}
    }

    if !ignore_security_fix {
        // TODO: Prevent using a seperate variable for unused security variable 
        let mut new_security = PAGE_PROTECTION_FLAGS::default();
        unsafe { let _ = VirtualProtectEx(
            process,
            module.0,
            data.len(),
            old_security,
            &mut new_security as *mut _,
        ); };
    }
    Ok(())
}

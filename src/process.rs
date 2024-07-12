use crate::errors::*;
use std::{mem, str};
use windows::Win32::{
    Foundation::*,
    System::{ProcessStatus::*, Threading::*},
};

const DWORD_SIZE: usize = mem::size_of::<u32>();

/// # `libdopamine::process::wait_for_process`
/// Blocking function to wait for a process.
/// 
/// ## Arguments
/// - **`process_name: &str` ->** Name of the process to wait for.
/// 
/// ## Return values
/// **If `Ok` ->** `(process_pid: u32, process_handle: HANDLE)`
/// 
/// **If `Err` ->** `libdopamine::errors::DopamineError`
pub fn wait_for_process(process_name: &str) -> Result<(u32, HANDLE), DopamineError> {
    let mut count: u32 = 32;
    loop {
        let mut cb_needed: u32 = 0;
        let mut loaded_processes: Vec<u32>;
        loop {
            loaded_processes = vec![u32::default(); count as usize * DWORD_SIZE];
            match unsafe { EnumProcesses(
                loaded_processes.as_mut_ptr(),
                loaded_processes.len() as u32,
                &mut cb_needed,
            ) } {
                Err(_) => {
                    let error = unsafe { GetLastError() };
                    let error_msg = format!("process list query fail, EnumProcesses return {}", error.0);
                    return Err(DopamineError::new(&error_msg, ErrorType::ReadWriteError));
                }, _ => {}
            }
            if cb_needed < loaded_processes.len() as u32 {
                loaded_processes.truncate(cb_needed as usize / DWORD_SIZE); break
            };
            count *= 2;
        }
        for pid in loaded_processes {
            let proc = unsafe {
                if let Ok(res) = OpenProcess(PROCESS_ALL_ACCESS, false, pid)
                { res } else { continue }
            };
            let mut proc_name: [u8; MAX_PATH as usize] = [0 as u8; MAX_PATH as usize];
            match unsafe {
                GetProcessImageFileNameA(proc, &mut proc_name as &mut [u8]) } {
                0 => continue, _ => {}
            }
            let proc_name_str: &str = std::str::from_utf8(&proc_name[..]).unwrap().trim_end_matches("\0");
            if proc_name_str.split("\\").last().unwrap() == process_name {
                return Ok((pid, proc));
            } else {
                unsafe { if let Err(_) = CloseHandle(proc) {} };
            }
        }
    }
}

/// # `libdopamine::process::close_process_handle`
/// Blocking function to wait for a process.
/// Usually safe to ignore when error is returned.
///  
/// ## Arguments
/// - **`process: HANDLE` ->** Process handle.
/// 
/// ## Return values
/// **If `Ok` ->** `()`
/// 
/// **If `Err` ->** `libdopamine::errors::DopamineError`
pub fn close_process_handle(process: HANDLE) -> Result <(), DopamineError> {
    match unsafe { CloseHandle(process) } {
        Err(_) => {
            let error = unsafe { GetLastError() };
            let error_msg = format!("couldn't close process handle, CloseHandle return {}", error.0);
            return Err(DopamineError::new(&error_msg, ErrorType::ReadWriteError));
        }, _ => Ok(())
    }
}

use std::ffi::c_void;
use std::mem::size_of;

pub const PROCESS_SET_QUOTA: u32 = 0x0100;
pub const PROCESS_QUERY_INFORMATION: u32 = 0x0400;

pub const TOKEN_ADJUST_PRIVILEGES: u32 = 0x0020;
pub const TOKEN_QUERY: u32 = 0x0008;
pub const SE_PRIVILEGE_ENABLED: u32 = 0x00000002;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LUID {
    pub low_part: u32,
    pub high_part: i32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LUID_AND_ATTRIBUTES {
    pub luid: LUID,
    pub attributes: u32,
}

#[repr(C)]
pub struct TOKEN_PRIVILEGES {
    pub privilege_count: u32,
    pub privileges: [LUID_AND_ATTRIBUTES; 1],
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn OpenProcess(
        dwDesiredAccess: u32,
        bInheritHandle: i32,
        dwProcessId: u32,
    ) -> *mut c_void;
    pub fn CloseHandle(hObject: *mut c_void) -> i32;
    pub fn GetCurrentProcess() -> *mut c_void;
    pub fn SetProcessWorkingSetSize(
        hProcess: *mut c_void,
        dwMinimumWorkingSetSize: usize,
        dwMaximumWorkingSetSize: usize,
    ) -> i32;
    pub fn K32EmptyWorkingSet(hProcess: *mut c_void) -> i32;
}

#[link(name = "advapi32")]
unsafe extern "system" {
    pub fn OpenProcessToken(
        ProcessHandle: *mut c_void,
        DesiredAccess: u32,
        TokenHandle: *mut *mut c_void,
    ) -> i32;
    pub fn LookupPrivilegeValueW(
        lpSystemName: *const u16,
        lpName: *const u16,
        lpLuid: *mut LUID,
    ) -> i32;
    pub fn AdjustTokenPrivileges(
        TokenHandle: *mut c_void,
        DisableAllPrivileges: i32,
        NewState: *const TOKEN_PRIVILEGES,
        BufferLength: u32,
        PreviousState: *mut c_void,
        ReturnLength: *mut u32,
    ) -> i32;
}

#[link(name = "ntdll")]
unsafe extern "system" {
    pub fn NtSetSystemInformation(
        SystemInformationClass: u32,
        SystemInformation: *const c_void,
        SystemInformationLength: u32,
    ) -> i32;
}

/// Enable a specific Windows token privilege (e.g. SeProfileSingleProcessPrivilege)
pub fn enable_privilege(privilege_name: &str) -> bool {
    unsafe {
        let mut token: *mut c_void = std::ptr::null_mut();
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == 0
        {
            return false;
        }

        let wide_name: Vec<u16> = privilege_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut luid = LUID {
            low_part: 0,
            high_part: 0,
        };

        if LookupPrivilegeValueW(std::ptr::null(), wide_name.as_ptr(), &mut luid) == 0 {
            CloseHandle(token);
            return false;
        }

        let tp = TOKEN_PRIVILEGES {
            privilege_count: 1,
            privileges: [LUID_AND_ATTRIBUTES {
                luid,
                attributes: SE_PRIVILEGE_ENABLED,
            }],
        };

        let result = AdjustTokenPrivileges(
            token,
            0,
            &tp,
            size_of::<TOKEN_PRIVILEGES>() as u32,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        CloseHandle(token);
        result != 0
    }
}

/// Purges the Windows Standby List (Reclaims cached memory held by OS file cache)
/// SystemInformationClass = 80 (SystemMemoryListInformation)
/// Command = 4 (MemoryPurgeStandbyList)
pub fn purge_standby_list() -> Result<bool, String> {
    enable_privilege("SeProfileSingleProcessPrivilege");
    enable_privilege("SeIncreaseQuotaPrivilege");

    let command: u32 = 4; // MemoryPurgeStandbyList
    let status = unsafe {
        NtSetSystemInformation(
            80, // SystemMemoryListInformation
            &command as *const u32 as *const c_void,
            size_of::<u32>() as u32,
        )
    };

    if status == 0 {
        Ok(true)
    } else {
        Err(format!("NTSTATUS error code: 0x{:08X} (通常需以系統管理員身分執行)", status))
    }
}

/// Flush memory working sets globally (SystemMemoryListInformation command 2 = MemoryEmptyWorkingSets)
pub fn empty_system_working_sets() -> Result<bool, String> {
    enable_privilege("SeProfileSingleProcessPrivilege");
    enable_privilege("SeIncreaseQuotaPrivilege");

    let command: u32 = 2; // MemoryEmptyWorkingSets
    let status = unsafe {
        NtSetSystemInformation(
            80,
            &command as *const u32 as *const c_void,
            size_of::<u32>() as u32,
        )
    };

    if status == 0 {
        Ok(true)
    } else {
        Err(format!("NTSTATUS error code: 0x{:08X}", status))
    }
}

/// Trims the working set of a process by PID
pub fn trim_process_working_set(pid: u32) -> bool {
    unsafe {
        let handle = OpenProcess(
            PROCESS_SET_QUOTA | PROCESS_QUERY_INFORMATION,
            0,
            pid,
        );
        if handle.is_null() {
            return false;
        }

        // Try K32EmptyWorkingSet first
        let ret = K32EmptyWorkingSet(handle);
        // Also call SetProcessWorkingSetSize with -1 to purge all pages
        SetProcessWorkingSetSize(handle, usize::MAX, usize::MAX);

        CloseHandle(handle);
        ret != 0
    }
}

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{LauncherError, Result};

/// Disk space information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskSpaceInfo {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub available_gb: f64,
    pub total_gb: f64,
}

/// Get disk space information for a path
pub fn get_disk_space(path: &PathBuf) -> Result<DiskSpaceInfo> {
    #[cfg(target_os = "windows")]
    {
        get_disk_space_windows(path)
    }
    
    #[cfg(target_os = "macos")]
    {
        get_disk_space_unix(path)
    }
    
    #[cfg(target_os = "linux")]
    {
        get_disk_space_unix(path)
    }
}

#[cfg(target_os = "windows")]
fn get_disk_space_windows(path: &PathBuf) -> Result<DiskSpaceInfo> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    
    // Get the root path (drive letter)
    let root = if let Some(prefix) = path.components().next() {
        PathBuf::from(prefix.as_os_str())
    } else {
        return Err(LauncherError::FileSystem("Invalid path".to_string()));
    };
    
    // Convert to wide string for Windows API
    let mut root_wide: Vec<u16> = OsStr::new(&root)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    
    let mut free_bytes_available: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free_bytes: u64 = 0;
    
    unsafe {
        use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
        use windows::core::PCWSTR;
        
        let result = GetDiskFreeSpaceExW(
            PCWSTR::from_raw(root_wide.as_ptr()),
            Some(&mut free_bytes_available),
            Some(&mut total_bytes),
            Some(&mut total_free_bytes),
        );
        
        if result.is_err() {
            return Err(LauncherError::FileSystem(
                "Failed to get disk space information".to_string()
            ));
        }
    }
    
    let used_bytes = total_bytes.saturating_sub(total_free_bytes);
    
    Ok(DiskSpaceInfo {
        total_bytes,
        available_bytes: free_bytes_available,
        used_bytes,
        available_gb: free_bytes_available as f64 / 1_073_741_824.0,
        total_gb: total_bytes as f64 / 1_073_741_824.0,
    })
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn get_disk_space_unix(path: &PathBuf) -> Result<DiskSpaceInfo> {
    use std::ffi::CString;
    use std::mem;
    
    let path_cstr = CString::new(path.to_string_lossy().as_bytes())
        .map_err(|e| LauncherError::FileSystem(format!("Invalid path: {}", e)))?;
    
    unsafe {
        let mut stat: libc::statvfs = mem::zeroed();
        if libc::statvfs(path_cstr.as_ptr(), &mut stat) != 0 {
            return Err(LauncherError::FileSystem(
                "Failed to get disk space information".to_string()
            ));
        }
        
        let block_size = stat.f_frsize as u64;
        let total_bytes = stat.f_blocks * block_size;
        let available_bytes = stat.f_bavail * block_size;
        let used_bytes = total_bytes.saturating_sub(stat.f_bfree * block_size);
        
        Ok(DiskSpaceInfo {
            total_bytes,
            available_bytes,
            used_bytes,
            available_gb: available_bytes as f64 / 1_073_741_824.0,
            total_gb: total_bytes as f64 / 1_073_741_824.0,
        })
    }
}

/// Check if there's enough disk space for installation
pub fn has_enough_space(path: &PathBuf, required_bytes: u64) -> Result<bool> {
    let disk_info = get_disk_space(path)?;
    Ok(disk_info.available_bytes >= required_bytes)
}

/// Format bytes to human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

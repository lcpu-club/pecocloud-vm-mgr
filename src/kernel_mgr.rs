use std::{env, io::Read, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{VmManageError, VmManageResult};


#[derive(Debug, Clone, Serialize, Deserialize)]
struct KernelItem {
    kernel_name: String,
    kernel_version: String,
    path: String,
}

// Get kernel image paths from a configuration file
// re-get every time to ensure dynamic modification
pub fn get_kernel_image_path(kernel_name: &String, kernel_version: &String) -> VmManageResult<PathBuf> {
    let kernel_list_file = env::var("KERNEL_LIST_FILE")
        .map_err(|_| VmManageError::EnvKernelList)?;
    let mut f = std::fs::File::open(kernel_list_file)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;

    let list: Vec<KernelItem> = serde_json::from_str(&buf)?;
    let mut path: Option<String> = None;
    for e in &list {
        if &e.kernel_name == kernel_name && &e.kernel_version == kernel_version {
            path = Some(e.path.to_owned());
            break;
        }
    }
    match path {
        Some(path) => Ok(PathBuf::from(path)),
        None => Err(VmManageError::KernelNotFound(kernel_name.to_owned())),
    }
}

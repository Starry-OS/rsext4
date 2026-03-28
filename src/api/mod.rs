//! High-level filesystem API exports.

use crate::BLOCK_SIZE;
use crate::blockdev::*;
use crate::dir::*;
use crate::error::*;
use crate::ext4::*;
use crate::file::*;
use crate::loopfile::*;
use crate::*;
use alloc::vec::Vec;

mod file_handle;
mod fs;
mod io;

pub use file_handle::OpenFile;
pub use fs::{fs_mount, fs_umount};
pub use io::{lseek, open, read, read_at, write_at};

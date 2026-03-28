//! Core filesystem state, mount, allocation, and mkfs helpers.

use ::alloc::collections::VecDeque;
use ::alloc::vec::Vec;
use log::trace;
use log::{debug, error, info, warn};

use crate::bitmap::InodeBitmap;
use crate::blockdev::*;
use crate::blockgroup_description::*;
use crate::bmalloc::*;
use crate::cache::bitmap::CacheKey;
use crate::cache::*;
use crate::checksum::*;
use crate::config::*;
use crate::crc32c::ext4_superblock_has_metadata_csum;
use crate::dir::*;
use crate::disknode::*;
use crate::endian::*;
use crate::error::*;
use crate::jbd2::jbd2::*;
use crate::jbd2::jbdstruct::*;
use crate::loopfile::*;
use crate::superblock::*;
use crate::tool::*;

mod alloc;
mod fs;
mod lookup;
mod mkfs;
mod mount;
mod sync;

pub use fs::{Ext4FileSystem, FileSystemStats};
pub use lookup::{file_entry_exisr, find_file};
pub use mkfs::{BlcokGroupLayout, FsLayoutInfo, compute_fs_layout, mkfs};
pub use mount::mount;
pub use sync::umount;

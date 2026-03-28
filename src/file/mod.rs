//! File and inode data operations.

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use log::{debug, error, info, warn};

use crate::blockdev::*;
use crate::bmalloc::{AbsoluteBN, InodeNumber};
use crate::checksum::update_ext4_dirblock_csum32;
use crate::config::*;
use crate::dir::*;
use crate::disknode::*;
use crate::entries::*;
use crate::error::*;
use crate::ext4::*;
use crate::extents_tree::*;
use crate::loopfile::*;
use crate::metadata::{Ext4DtimeUpdate, Ext4InodeMetadataUpdate};
use crate::superblock::Ext4Superblock;

mod blocks;
mod create;
mod delete;
mod io;
mod link;
mod rename;

pub use blocks::build_file_block_mapping;
pub use create::{create_symbol_link, mkfile};
pub use delete::{delete_dir, delete_file, unlink};
pub use io::{read_file, truncate, write_file};
pub use link::link;
pub use rename::{mv, rename};

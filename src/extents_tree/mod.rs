//! Extent tree node parsing and update helpers.

use alloc::vec;
use alloc::vec::*;
use log::{debug, error};

use crate::blockdev::*;
use crate::bmalloc::AbsoluteBN;
use crate::config::*;
use crate::disknode::*;
use crate::endian::*;
use crate::error::*;
use crate::ext4::*;

mod insert;
mod node;
mod parse;
mod remove;
mod root;
mod split;

pub use node::ExtentNode;
pub use root::ExtentTree;

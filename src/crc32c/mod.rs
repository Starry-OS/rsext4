pub mod arm64;

#[allow(clippy::module_inception)]
pub mod crc32c;

// Re-export commonly used functions from crc32c module
pub use crc32c::crc32c;
pub use crc32c::crc32c_append;
pub use crc32c::crc32c_finalize;
pub use crc32c::crc32c_init;
pub use crc32c::ext4_crc32c_seed_from_superblock;
pub use crc32c::ext4_superblock_has_metadata_csum;

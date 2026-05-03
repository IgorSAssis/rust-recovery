/// Number of bytes in one gibibyte (1024³).
pub(crate) const BYTES_PER_GIB: u64 = 1024 * 1024 * 1024;

/// Number of bytes per logical block as reported by the Linux kernel in `/sys/block/<dev>/size`.
pub(crate) const BYTES_PER_BLOCK: u64 = 512;

/// Path to the sysfs block device directory.
pub(crate) const SYS_BLOCK_PATH: &str = "/sys/block";

/// Prefix for raw device paths.
pub(crate) const DEV_PATH_PREFIX: &str = "/dev";

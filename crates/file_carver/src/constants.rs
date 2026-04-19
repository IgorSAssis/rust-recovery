/// Default number of bytes transferred per iteration for both scanning and
/// extraction operations. 4 KiB aligns with the most common OS page size and
/// is a good balance between memory usage and syscall overhead.
pub const DEFAULT_CHUNK_SIZE: usize = 4096;

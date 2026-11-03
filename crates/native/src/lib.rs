//! User-process contracts only. No OS backend, process attachment or unsafe code.
mod memory;
mod target;
pub use memory::{
    MEMORY_READ, MEMORY_WRITE, MemoryError, MemoryRead, MemoryResult, MemoryWrite, NativeSession,
    read_memory, write_memory,
};
pub use target::{ByteOrder, NativeTarget, TargetAddress, TargetArchitecture};

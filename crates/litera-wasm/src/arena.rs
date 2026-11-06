//! Deterministic Bump Arena Allocator for WebAssembly Linear Memory (`wasm32-unknown-unknown`).
//!
//! Provides O(1) pointer-bump allocations with 8-byte alignment and single-instruction
//! bulk reset between manuscript analysis frames, avoiding general-purpose `dlmalloc`/`wee_alloc`
//! fragmentation in real-time keystroke loops.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArenaSliceHandle {
    pub offset: usize,
    pub len: usize,
}

pub struct BumpArena {
    buffer: Vec<u8>,
    cursor: usize,
    high_water_mark: usize,
    allocation_count: usize,
}

impl BumpArena {
    /// Creates a new `BumpArena` with the specified byte capacity (e.g., 64 KiB = 1 WASM page).
    pub fn with_capacity(capacity_bytes: usize) -> Self {
        let aligned_cap = (capacity_bytes.max(1024) + 7) & !7;
        Self {
            buffer: vec![0u8; aligned_cap],
            cursor: 0,
            high_water_mark: 0,
            allocation_count: 0,
        }
    }

    /// Allocates an 8-byte aligned slice of `size` bytes in O(1) time.
    /// Returns `None` if the arena capacity would be exceeded.
    #[inline]
    pub fn alloc_bytes(&mut self, size: usize) -> Option<ArenaSliceHandle> {
        let aligned_cursor = (self.cursor + 7) & !7;
        let next_cursor = aligned_cursor.checked_add(size)?;
        if next_cursor > self.buffer.len() {
            return None;
        }
        self.cursor = next_cursor;
        if self.cursor > self.high_water_mark {
            self.high_water_mark = self.cursor;
        }
        self.allocation_count += 1;
        Some(ArenaSliceHandle {
            offset: aligned_cursor,
            len: size,
        })
    }

    /// Copies a UTF-8 string into the arena and returns its handle.
    #[inline]
    pub fn push_str(&mut self, text: &str) -> Option<ArenaSliceHandle> {
        let bytes = text.as_bytes();
        let handle = self.alloc_bytes(bytes.len())?;
        self.buffer[handle.offset..handle.offset + handle.len].copy_from_slice(bytes);
        Some(handle)
    }

    /// Resolves an `ArenaSliceHandle` back to a zero-copy `&str` slice.
    #[inline]
    pub fn get_str(&self, handle: ArenaSliceHandle) -> Option<&str> {
        let slice = self.buffer.get(handle.offset..handle.offset + handle.len)?;
        core::str::from_utf8(slice).ok()
    }

    /// Resets the arena cursor to zero in O(1) time without freeing the backing WASM page.
    #[inline]
    pub fn reset(&mut self) {
        self.cursor = 0;
        self.allocation_count = 0;
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    pub fn used_bytes(&self) -> usize {
        self.cursor
    }

    #[inline]
    pub fn high_water_mark(&self) -> usize {
        self.high_water_mark
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bump_arena_alignment_and_reset() {
        let mut arena = BumpArena::with_capacity(4096);
        let h1 = arena.push_str("Тишина").expect("alloc 1");
        let h2 = arena.push_str("Литера").expect("alloc 2");
        assert_eq!(h1.offset % 8, 0);
        assert_eq!(h2.offset % 8, 0);
        assert_eq!(arena.get_str(h1), Some("Тишина"));
        assert_eq!(arena.get_str(h2), Some("Литера"));
        arena.reset();
        assert_eq!(arena.used_bytes(), 0);
        assert!(arena.high_water_mark() > 0);
    }
}

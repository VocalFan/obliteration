use core::alloc::Layout;
use core::ptr::{null_mut, NonNull};
use talc::{ClaimOnOom, Span, Talc};

/// Stage 1 kernel heap.
///
/// This stage allocate a memory from a static buffer (AKA arena).
pub struct Stage1 {
    engine: spin::Mutex<Talc<ClaimOnOom>>,
    buf_ptr: *const u8,
    buf_end: *const u8,
}

impl Stage1 {
    /// # Safety
    /// The specified memory must be valid for reads and writes and it must be exclusively available
    /// to [`Stage1`].
    pub const unsafe fn new(buf: *mut u8, len: usize) -> Self {
        let engine = Talc::new(ClaimOnOom::new(Span::from_base_size(buf, len)));

        Self {
            engine: spin::Mutex::new(engine),
            buf_ptr: buf,
            buf_end: buf.add(len),
        }
    }

    pub fn is_owner(&self, ptr: *const u8) -> bool {
        ptr >= self.buf_ptr && ptr < self.buf_end
    }

    /// The returned pointer will always within the buffer that was specified in the
    /// [`Self::new()`].
    ///
    /// # Safety
    /// `layout` must be nonzero.
    pub unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.engine
            .lock()
            .malloc(layout)
            .map(|v| v.as_ptr())
            .unwrap_or(null_mut())
    }

    /// # Safety
    /// `ptr` must be obtained with [`Self::alloc()`] and `layout` must be the same one that was
    /// passed to that method.
    pub unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.engine.lock().free(NonNull::new_unchecked(ptr), layout);
    }
}
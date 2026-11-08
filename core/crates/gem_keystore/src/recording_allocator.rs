use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

pub(crate) type FreedMatch = fn(Layout, &[u8]) -> bool;

#[global_allocator]
static ALLOCATOR: RecordingAllocator = RecordingAllocator;

thread_local! {
    static RECORDING: Recording = const { Recording { matches: Cell::new(None), matched: Cell::new(0) } };
}

pub(crate) struct RecordingAllocator;

impl RecordingAllocator {
    pub(crate) fn count_freed(matches: FreedMatch, run: impl FnOnce()) -> usize {
        RECORDING.with(|recording| {
            recording.matched.set(0);
            recording.matches.set(Some(matches));
        });
        run();
        RECORDING.with(|recording| {
            recording.matches.set(None);
            recording.matched.get()
        })
    }
}

unsafe impl GlobalAlloc for RecordingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if RECORDING.try_with(|recording| recording.matches.get().is_some()).unwrap_or(false) {
            unsafe { System.alloc_zeroed(layout) }
        } else {
            unsafe { System.alloc(layout) }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let _ = RECORDING.try_with(|recording| unsafe { recording.inspect(ptr, layout) });
        unsafe { System.dealloc(ptr, layout) }
    }
}

struct Recording {
    matches: Cell<Option<FreedMatch>>,
    matched: Cell<usize>,
}

impl Recording {
    /// Safety: `ptr` and `layout` describe a live block. While recording, every block comes from `alloc_zeroed`, so all of its bytes are initialized.
    unsafe fn inspect(&self, ptr: *const u8, layout: Layout) {
        let Some(matches) = self.matches.get() else { return };
        if matches(layout, unsafe { std::slice::from_raw_parts(ptr, layout.size()) }) {
            self.matched.set(self.matched.get() + 1);
        }
    }
}

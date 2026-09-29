//! Public allocator adapter: no JS calls/reentry and no private layouts.
#![allow(unsafe_code)]
use crate::{Phase, State};
use rquickjs::allocator::{Allocator, RustAllocator};
use std::{cell::Cell, ptr, rc::Rc};

#[derive(Default)]
pub struct Accounting {
    pub live: Cell<usize>,
    pub peak: Cell<usize>,
    pub calls: Cell<usize>,
    pub rejects: Cell<usize>,
    pub single: Cell<usize>,
    pub total: Cell<usize>,
    pub fail: Cell<Option<(Phase, usize)>>,
    pub phase_calls: Cell<usize>,
}
pub struct Tracked(pub Rc<Accounting>, pub Rc<State>);
impl Tracked {
    fn allow(&self, size: usize, old: usize) -> bool {
        let a = &self.0;
        a.calls.set(a.calls.get() + 1);
        let mut injected = false;
        if let Some((phase, n)) = a.fail.get()
            && self.1.phase.get() == phase
        {
            let seen = a.phase_calls.get() + 1;
            a.phase_calls.set(seen);
            injected = seen == n;
        }
        // Reserve conservative rounding/metadata headroom; this is adapter
        // accounting, not RSS. Reject arithmetic overflow before RustAllocator.
        let predicted = size
            .checked_add(128)
            .and_then(|s| a.live.get().checked_sub(old)?.checked_add(s));
        let over = predicted.is_none_or(|n| a.total.get() != 0 && n > a.total.get());
        if injected || over || (a.single.get() != 0 && size > a.single.get()) {
            a.rejects.set(a.rejects.get() + 1);
            self.1.latch(admission::Reason::ResourceLimit);
            false
        } else {
            true
        }
    }
    fn update(&self, old: usize, new: usize) {
        let live = self.0.live.get() - old + new;
        self.0.live.set(live);
        self.0.peak.set(self.0.peak.get().max(live));
    }
    unsafe fn charge(p: *mut u8) -> usize {
        if p.is_null() {
            0
        } else {
            // SAFETY: non-null pointer from this exact RustAllocator delegate.
            unsafe { RustAllocator::usable_size(p) + 64 }
        }
    }
}
// SAFETY: all successful allocations, alignment, usable-size and frees delegate
// to one RustAllocator. Refusal returns null without consuming an existing block.
// No allocator callback calls the VM, allocates tracking maps, or panics normally.
unsafe impl Allocator for Tracked {
    fn alloc(&mut self, size: usize) -> *mut u8 {
        if !self.allow(size, 0) {
            return ptr::null_mut();
        }
        let p = RustAllocator.alloc(size);
        if p.is_null() {
            self.1.latch(admission::Reason::ResourceLimit);
        }
        // SAFETY: successful delegate pointer, or null.
        self.update(0, unsafe { Self::charge(p) });
        p
    }
    fn calloc(&mut self, count: usize, size: usize) -> *mut u8 {
        if count == 0 || size == 0 {
            return ptr::null_mut();
        }
        let Some(total) = count.checked_mul(size) else {
            self.1.latch(admission::Reason::ResourceLimit);
            return ptr::null_mut();
        };
        if !self.allow(total, 0) {
            return ptr::null_mut();
        }
        let p = RustAllocator.calloc(count, size);
        if p.is_null() {
            self.1.latch(admission::Reason::ResourceLimit);
        }
        // SAFETY: successful delegate pointer, or null.
        self.update(0, unsafe { Self::charge(p) });
        p
    }
    unsafe fn dealloc(&mut self, p: *mut u8) {
        if p.is_null() {
            return;
        }
        // SAFETY: caller supplies a live allocation from this adapter.
        let old = unsafe { Self::charge(p) };
        unsafe { RustAllocator.dealloc(p) };
        self.update(old, 0);
    }
    unsafe fn realloc(&mut self, p: *mut u8, size: usize) -> *mut u8 {
        if p.is_null() {
            return self.alloc(size);
        }
        if size == 0 {
            // SAFETY: caller supplies its existing allocation.
            unsafe { self.dealloc(p) };
            return ptr::null_mut();
        }
        // SAFETY: caller's live delegate block remains valid on refusal.
        let old = unsafe { Self::charge(p) };
        if !self.allow(size, old) {
            return ptr::null_mut();
        }
        let next = unsafe { RustAllocator.realloc(p, size) };
        if next.is_null() {
            self.1.latch(admission::Reason::ResourceLimit);
        } else {
            self.update(old, unsafe { Self::charge(next) });
        }
        next
    }
    unsafe fn usable_size(p: *mut u8) -> usize {
        if p.is_null() {
            0
        } else {
            // SAFETY: public allocator contract supplies a live delegate block.
            unsafe { RustAllocator::usable_size(p) }
        }
    }
}

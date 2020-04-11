//! A local heap impl for [nanojpeg-rs](https://github.com/jonlamb-gh/nanojpeg-rs)
//!
//! My fork of linked_list_allocator uses core::alloc::Layout instead of the
//! alloc crate to avoid global allocator requirements, just need a heap to back
//! the nanojpeg-rs libc stuff.

use crate::hal::cache;
use core::alloc::Layout;
use core::{ptr, slice};
use heapless::consts::U256;
use heapless::LinearMap;
use linked_list_allocator::Heap;

const MIN_ALIGN: ctypes::size_t = 8;
//const MIN_ALIGN: ctypes::size_t = 16;

pub const HEAP_SIZE: usize = 320 * 240 * 4 * 5;

static mut MAP: LinearMap<usize, Layout, U256> = LinearMap(heapless::i::LinearMap::new());
pub static mut HEAP_MEM: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
pub static mut HEAP: Heap = Heap::empty();
static mut WMARK: usize = HEAP_SIZE;

#[allow(non_camel_case_types)]
mod ctypes {
    pub type size_t = usize;
    pub type c_int = i32;
    pub type c_void = core::ffi::c_void;
    pub type c_uchar = u8;
}

#[no_mangle]
pub unsafe extern "C" fn njAllocMem(size: ctypes::c_int) -> *mut ctypes::c_void {
    let size = size as usize;
    if size == 0 {
        return ptr::null_mut();
    }

    let size = roundup(size, MIN_ALIGN);
    let layout = layout_from_size_align(size as usize, MIN_ALIGN);
    let ptr = HEAP
        .allocate_first_fit(layout.clone())
        .ok()
        .map_or(0 as *mut u8, |allocation| allocation.as_ptr());
    if !ptr.is_null() {
        WMARK -= layout.size();
        insert_layout(ptr, layout);
        cache::clean_and_invalidate_data_cache_range(ptr as _, size);
    } else {
        panic!("Heap out of memory, WMARK = {}", WMARK);
    }
    ptr as *mut ctypes::c_void
}

#[no_mangle]
pub unsafe extern "C" fn njFreeMem(ptr: *mut ctypes::c_void) {
    if ptr.is_null() {
        return;
    }
    let layout = get_layout(ptr as *mut u8);
    WMARK += layout.size();
    delete_layout(ptr as *mut u8);
    HEAP.deallocate(ptr::NonNull::new_unchecked(ptr as *mut u8), layout);
}

#[no_mangle]
pub unsafe extern "C" fn njFillMem(
    block: *mut ctypes::c_void,
    byte: ctypes::c_uchar,
    size: ctypes::c_int,
) {
    if size > 0 {
        let slice = slice::from_raw_parts_mut(block as *mut u8, size as usize);
        slice.iter_mut().for_each(|b| *b = byte);
    }
}

#[no_mangle]
pub unsafe extern "C" fn njCopyMem(
    dst: *mut ctypes::c_void,
    src: *const ctypes::c_void,
    size: ctypes::c_int,
) {
    if size > 0 {
        let dst = slice::from_raw_parts_mut(dst as *mut u8, size as usize);
        let src = slice::from_raw_parts(src as *mut u8, size as usize);
        dst.copy_from_slice(src);
    }
}

#[inline(always)]
fn roundup(n: ctypes::size_t, multiple: ctypes::size_t) -> ctypes::size_t {
    if n == 0 {
        return multiple;
    }
    let remainder = n % multiple;
    if remainder == 0 {
        n
    } else {
        n + multiple - remainder
    }
}

#[inline(always)]
unsafe fn layout_from_size_align(size: usize, align: usize) -> Layout {
    if cfg!(debug_assertions) {
        Layout::from_size_align(size as usize, align).unwrap()
    } else {
        Layout::from_size_align_unchecked(size as usize, align)
    }
}

unsafe fn insert_layout(ptr: *mut u8, layout: Layout) {
    let _ = MAP.insert(ptr as usize, layout).expect("TODO");
}

unsafe fn get_layout(ptr: *mut u8) -> Layout {
    MAP.get(&(ptr as usize)).expect("TODO").clone()
}

unsafe fn delete_layout(ptr: *mut u8) {
    let _ = MAP.remove(&(ptr as usize)).expect("TODO");
}

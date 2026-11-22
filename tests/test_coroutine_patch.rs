// The archive this branch builds carries coroutine/coroutine-local.patch: every C
// export the patch adds links, and the patch version the archive reports is
// the one coroutine/coroutine-local.patch defines. Linking this binary at all
// proves the exports are present.

use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_void;

unsafe extern "C" {
  fn v8_coroutine_patch_version() -> c_int;
  fn v8_coroutine_slot_count() -> usize;
  fn v8_coroutine_slots_init(
    slots: *mut usize,
    stack_start: *mut c_void,
    stack_limit: *mut c_void,
  );
  fn v8_coroutine_slots_use(slots: *mut usize);
  fn v8_coroutine_microtask_count(isolate: *mut c_void) -> usize;
  fn v8_coroutine_microtask_peek(isolate: *mut c_void) -> isize;
  fn v8_coroutine_microtask_take(
    isolate: *mut c_void,
    to: *mut c_void,
    owner: *mut isize,
  ) -> c_int;
  fn v8_coroutine_microtask_owner(isolate: *mut c_void, owner: isize);
  fn v8_trace_start(categories: *const *const c_char, count: usize) -> c_int;
  fn v8_trace_stop();
  fn v8_trace_take(data: *mut *const c_char) -> usize;
  fn v8_trace_now_us() -> i64;
  fn v8_object_shape_new(
    isolate: *mut c_void,
    prototype_or_null: *const c_void,
    names: *const *const c_void,
    count: usize,
  ) -> *mut c_void;
  fn v8_object_shape_release(shape: *mut c_void);
  fn v8_object_shape_field_count(shape: *const c_void) -> usize;
  fn v8_object_shape_new_object(
    isolate: *mut c_void,
    shape: *const c_void,
    values: *const *const c_void,
    count: usize,
  ) -> *const c_void;
  fn v8_object_shape_new_objects(
    isolate: *mut c_void,
    shape: *const c_void,
    fields: *const u32,
    columns: usize,
    kinds: *const u8,
    numbers: *const f64,
    values: *const *const c_void,
    value_count: usize,
    rows: usize,
  ) -> *const c_void;
}

/// `V8_COROUTINE_PATCH_VERSION` as coroutine/coroutine-local.patch defines it.
fn patch_version_in_source() -> c_int {
  let patch = include_str!("../coroutine/coroutine-local.patch");
  patch
    .lines()
    .find_map(|line| line.strip_prefix("+#define V8_COROUTINE_PATCH_VERSION "))
    .expect("the patch defines V8_COROUTINE_PATCH_VERSION")
    .trim()
    .parse()
    .expect("V8_COROUTINE_PATCH_VERSION is a number")
}

#[test]
fn archive_carries_this_patch() {
  assert_eq!(
    unsafe { v8_coroutine_patch_version() },
    patch_version_in_source()
  );
  assert!(unsafe { v8_coroutine_slot_count() } > 0);
}

#[test]
fn clearing_the_current_slots_is_allowed() {
  unsafe { v8_coroutine_slots_use(std::ptr::null_mut()) };
}

#[test]
fn every_export_links() {
  let exports: [*const (); 17] = [
    v8_coroutine_patch_version as *const (),
    v8_coroutine_slot_count as *const (),
    v8_coroutine_slots_init as *const (),
    v8_coroutine_slots_use as *const (),
    v8_coroutine_microtask_count as *const (),
    v8_coroutine_microtask_peek as *const (),
    v8_coroutine_microtask_take as *const (),
    v8_coroutine_microtask_owner as *const (),
    v8_trace_start as *const (),
    v8_trace_stop as *const (),
    v8_trace_take as *const (),
    v8_trace_now_us as *const (),
    v8_object_shape_new as *const (),
    v8_object_shape_release as *const (),
    v8_object_shape_field_count as *const (),
    v8_object_shape_new_object as *const (),
    v8_object_shape_new_objects as *const (),
  ];
  assert!(exports.iter().all(|export| !export.is_null()));
}

#[test]
fn typed_arrays_up_to_64_bytes_live_on_the_heap() {
  assert_eq!(v8::TYPED_ARRAY_MAX_SIZE_IN_HEAP, 64);
}

#include <stdio.h>

typedef void (*rust_callback)(
  void*,
  const unsigned char*,
  unsigned long,
  unsigned long, 
  unsigned long,
  unsigned long);

void* cb_target;
rust_callback bridge_cb;


void trigger_callback(
  const unsigned char* pBuffer,
  unsigned long dwSize,
  unsigned long dwCount,
  unsigned long dwNotify,
  unsigned long dwUser
) {
  bridge_cb(cb_target, pBuffer, dwSize, dwCount, dwNotify, dwUser); // Will call callback(&rustObject, 7) in Rust.
}

void* make_bridge_callback(void* callback_target, rust_callback callback) {
    cb_target = callback_target;
    bridge_cb = callback;
    return trigger_callback;
}

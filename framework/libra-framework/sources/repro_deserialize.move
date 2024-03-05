
// reproducing an error condition in the rescue cli
// the Move VM when running a session aborts on evalutaing exists()==false
//  when it should only assign to false.
module ol_framework::repro_deserialize {

use diem_std::debug::print;

struct Noop has key {}

public fun should_not_abort() {
  // this causes a Dump since 0xabc does not exist yet.
  let a = exists<Noop>(@0xabc);
  print(&a);
}

public entry fun maybe_aborts(addr: address) {
  // this causes a Dump since 0xabc does not exist yet.
  let a = exists<Noop>(addr);
  print(&a);
}
}

extern crate usvg;

use std::mem;

#[test]
fn node_kind_size_1() {
    assert!(mem::size_of::<usvg::NodeKind>() < 256);
}

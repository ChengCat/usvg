extern crate usvg;

use std::mem;

use usvg::tree;

#[test]
fn node_kind_size_1() {
    assert!(mem::size_of::<tree::NodeKind>() < 256);
}

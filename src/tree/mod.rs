// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Implementation of the nodes tree.

use std::cell::Ref;

extern crate rctree;

// external
use svgdom;

// self
pub use self::node::*;
pub use self::attribute::*;

mod attribute;
mod dump;
mod node;

/// Basic modules and traits for tree manipulation.
pub mod prelude {
    pub use tree;
    pub use tree::FuzzyEq;
    pub use super::NodeExt;
}

/// Alias for `rctree::Node<Box<NodeKind>>`.
pub type Node = rctree::Node<Box<NodeKind>>;

/// A nodes tree container.
pub struct Tree {
    root: Node,
}

impl Tree {
    /// Creates a new `Tree`.
    pub fn create(svg: Svg) -> Self {
        let root_node = Node::new(Box::new(NodeKind::Svg(svg)));
        let defs_node = Node::new(Box::new(NodeKind::Defs));
        root_node.append(defs_node);

        Tree {
            root: root_node,
        }
    }

    /// Returns the `Svg` node.
    pub fn root(&self) -> Node {
        self.root.clone()
    }

    /// Returns the `Svg` node value.
    pub fn svg_node(&self) -> Ref<Svg> {
        Ref::map(self.root.borrow(), |v| {
            match **v {
                NodeKind::Svg(ref svg) => svg,
                _ => unreachable!(),
            }
        })
    }

    /// Returns the `Defs` node.
    pub fn defs(&self) -> Node {
        self.root.first_child().unwrap()
    }

    /// Checks that `node` is part of the `Defs` children.
    pub fn is_in_defs(&self, node: &Node) -> bool {
        let defs = self.defs();
        node.ancestors().any(|n| n == defs)
    }

    /// Appends `NodeKind` to the `Defs` node.
    pub fn append_to_defs(&mut self, kind: NodeKind) -> Node {
        let new_node = Node::new(Box::new(kind));
        self.defs().append(new_node.clone());
        new_node
    }

    /// Searches for `NodeId` in `Defs` children by ID.
    pub fn defs_by_id(&self, id: &str) -> Option<Node> {
        for n in self.defs().children() {
            if &*n.id() == id {
                return Some(n);
            }
        }

        warn!("Node '{}' was not found in defs.", id);
        None
    }

    /// Returns renderable node by ID.
    ///
    /// If an empty ID is provided, than this method will always return `None`.
    /// Even if tree has nodes with empty ID.
    pub fn node_by_svg_id(&self, id: &str) -> Option<Node> {
        if id.is_empty() {
            return None;
        }

        for node in self.root().descendants() {
            if !self.is_in_defs(&node) {
                if &*node.id() == id {
                    return Some(node);
                }
            }
        }

        None
    }

    /// Converts the document to `svgdom::Document`.
    ///
    /// Used to save document to file for debug purposes.
    pub fn to_svgdom(&self) -> svgdom::Document {
        dump::conv_doc(self)
    }
}

/// Additional `Node` methods.
pub trait NodeExt {
    /// Returns node's ID.
    ///
    /// If a current node doesn't support ID - an empty string
    /// will be returned.
    fn id(&self) -> Ref<str>;

    /// Returns node's transform.
    ///
    /// If a current node doesn't support transformation - a default
    /// transform will be returned.
    fn transform(&self) -> Transform;

    /// Returns `NodeKind` instead of `Box<NodeKind>`.
    ///
    /// Use it instead of `Node::value()`.
    fn kind(&self) -> Ref<NodeKind>;


    /// Appends `kind` as a node child.
    ///
    /// Shorthand for `Node::append(Node::new(Box::new(kind)))`.
    fn append_kind(&self, kind: NodeKind) -> Node;

    /// Returns a node's tree.
    fn tree(&self) -> Tree;
}

impl NodeExt for Node {
    fn id(&self) -> Ref<str> {
        Ref::map(self.borrow(), |v| v.id())
    }

    fn transform(&self) -> Transform {
        self.borrow().transform()
    }

    fn kind(&self) -> Ref<NodeKind> {
        Ref::map(self.borrow(), |v| &**v)
    }

    fn append_kind(&self, kind: NodeKind) -> Node {
        let new_node = Node::new(Box::new(kind));
        self.append(new_node.clone());
        new_node
    }

    fn tree(&self) -> Tree {
        Tree { root: self.root() }
    }
}

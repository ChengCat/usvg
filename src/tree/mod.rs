// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Implementation of the nodes tree.

extern crate ego_tree;

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
    pub use super::TreeExt;
}

/// Alias for `ego_tree::NodeId<NodeKind>`.
pub type NodeId = ego_tree::NodeId<NodeKind>;

/// Alias for `ego_tree::NodeRef<NodeKind>`.
pub type NodeRef<'a> = ego_tree::NodeRef<'a, NodeKind>;

/// Alias for `ego_tree::NodeMut<NodeKind>`.
pub type NodeMut<'a> = ego_tree::NodeMut<'a, NodeKind>;

/// A nodes tree container.
///
/// Alias for `ego_tree::Tree<NodeKind>`.
pub type Tree = ego_tree::Tree<NodeKind>;

/// Additional `Tree` methods.
pub trait TreeExt {
    /// Creates a new `Tree`.
    fn create(svg: Svg) -> Self;

    /// Returns the `Svg` node's data.
    fn svg_node(&self) -> &Svg;

    /// Returns the `Defs` node.
    fn defs(&self) -> NodeRef;

    /// Checks that `node` is part of the `Defs` children.
    fn is_in_defs(&self, node: NodeRef) -> bool;

    /// Appends `NodeKind` to the `Defs` node.
    fn append_to_defs(&mut self, kind: NodeKind) -> NodeId;

    /// Append `NodeKind` as a child to the node by `NodeId`.
    fn append_child(&mut self, parent: NodeId, kind: NodeKind) -> NodeId;

    /// Returns `Defs` node child by `NodeId`.
    fn defs_at(&self, id: NodeId) -> Option<NodeRef>;

    /// Searches for `NodeId` in `Defs` children by ID.
    fn defs_by_svg_id(&self, id: &str) -> Option<NodeId>;

    /// Returns renderable node by ID.
    ///
    /// If an empty ID is provided, than this method will always return `None`.
    /// Even if tree has nodes with empty ID.
    fn node_by_svg_id(&self, id: &str) -> Option<NodeRef>;

    /// Converts the document to `svgdom::Document`.
    ///
    /// Used to save document to file for debug purposes.
    fn to_svgdom(&self) -> svgdom::Document;
}

impl TreeExt for Tree {
    fn create(svg: Svg) -> Self {
        let mut tree = ego_tree::Tree::new(NodeKind::Svg(svg));
        tree.root_mut().append(NodeKind::Defs);
        tree
    }

    fn svg_node(&self) -> &Svg {
        if let NodeKind::Svg(ref svg) = *self.root().value() {
            svg
        } else {
            unreachable!();
        }
    }

    fn defs(&self) -> NodeRef {
        self.root().first_child().unwrap()
    }

    fn is_in_defs(&self, node: NodeRef) -> bool {
        let defs = self.defs();
        node.ancestors().any(|n| n == defs)
    }

    fn append_to_defs(&mut self, kind: NodeKind) -> NodeId {
        let defs_id = self.defs().id();
        self.append_child(defs_id, kind)
    }

    fn append_child(&mut self, parent: NodeId, kind: NodeKind) -> NodeId {
        let mut parent = self.get_mut(parent);
        parent.append(kind).id()
    }

    fn defs_at(&self, id: NodeId) -> Option<NodeRef> {
        for n in self.defs().children() {
            if n.id() == id {
                return Some(n);
            }
        }

        warn!("Node was not found in defs.");
        None
    }

    fn defs_by_svg_id(&self, id: &str) -> Option<NodeId> {
        for n in self.defs().children() {
            if n.svg_id() == id {
                return Some(n.id());
            }
        }

        warn!("Node '{}' was not found in defs.", id);
        None
    }

    fn node_by_svg_id(&self, id: &str) -> Option<NodeRef> {
        if id.is_empty() {
            return None;
        }

        for node in self.root().descendants() {
            if !self.is_in_defs(node) {
                if node.svg_id() == id {
                    return Some(node);
                }
            }
        }

        None
    }

    fn to_svgdom(&self) -> svgdom::Document {
        dump::conv_doc(self)
    }
}

/// Additional `NodeRef` methods.
pub trait NodeExt {
    /// Returns node's ID.
    ///
    /// If a current node doesn't support ID - an empty string
    /// will be returned.
    fn svg_id(&self) -> &str;

    /// Returns node's transform.
    ///
    /// If a current node doesn't support transformation - a default
    /// transform will be returned.
    fn transform(&self) -> Transform;
}

impl<'a> NodeExt for NodeRef<'a> {
    fn svg_id(&self) -> &str {
        self.value().id()
    }

    fn transform(&self) -> Transform {
        self.value().transform()
    }
}

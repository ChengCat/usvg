# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- Remove elements with `opacity="0"`.
- Transfer the group `id` attribute to the child when group has only one child.
- `symbol` element support.
- `Tree::from_str`.
- Nested `svg` elements support.

### Changed
- `tree` module content reexported.
- `parse_tree_from_*` methods move to the `Tree`. Use `Tree::from_*` instead.
- Rename `Tree::node_by_svg_id` to `Tree::node_by_id`.
- Use `rctree::Node<NodeKind>` instead of `rctree::Node<Box<NodeKind>>`.
- `view` element is out of scope now.
- `FileReadError` replaced with `Error`.
- `parse_tree_from_data` accepts `&[u8]` and not `&str` now.

### Removed
- `NodeExt::kind`. Use `Node::borrow` instead.

### Fixed
- Panic during `visibility` resolving.
- Gradients with one stop resolving.
- `use` attributes resolving.
- `clipPath` and `mask` attributes resolving.
- `offset` attribute in `stop` element resolving.
- Incorrect `font-size` attribute resolving.

[Unreleased]: https://github.com/RazrFalcon/usvg/compare/v0.1.1...HEAD

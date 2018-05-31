# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- Implement `Deref` for `LinearGradient` and `RadialGradient`.

### Changed
- Gradient stops are stored directly in the `BaseGradient` and not as `NodeKind::Stop` now.
- `TextChunk` are stored directly in the `Text` and not as `NodeKind::TextChunk` now.
- Rename `LinearGradient::d` into `LinearGradient::base`.
- Rename `RadialGradient::d` into `RadialGradient::base`.

### Removed
- `failure` dependency.

## [0.2.0] - 2018-05-23
### Added
- Remove elements with `opacity="0"`.
- Transfer the group `id` attribute to the child when group has only one child.
- `symbol` element support.
- `Tree::from_str`.
- Nested `svg` elements support.
- SVG support for `image` element.
- `ImageFormat::SVG`.
- `Image::format`.
- Paint fallback resolving.
- Bbox validation for shapes that use painting servers.
- `TextChunk::dx` and `TextChunk::dy`.
- `Text::rotate`.
- `rotate` attribute processing.

### Changed
- `tree` module content reexported.
- `parse_tree_from_*` methods move to the `Tree`. Use `Tree::from_*` instead.
- Rename `Tree::node_by_svg_id` to `Tree::node_by_id`.
- Use `rctree::Node<NodeKind>` instead of `rctree::Node<Box<NodeKind>>`.
- `view` element is out of scope now.
- `FileReadError` replaced with `Error`.
- `parse_tree_from_data` accepts `&[u8]` and not `&str` now.
- Rename `ImageDataKind` to `ImageFormat`.
- New geometry primitives implementation.
- `TextChunk::x` and `TextChunk::y` are `Option<NumberList>` and not `f64` now.

### Removed
- `NodeExt::kind`. Use `Node::borrow` instead.

### Fixed
- Panic during `visibility` resolving.
- Gradients with one stop resolving.
- `use` attributes resolving.
- `clipPath` and `mask` attributes resolving.
- `offset` attribute in `stop` element resolving.
- Incorrect `font-size` attribute resolving.
- Gradient stops resolving.
- `switch` element resolving.

[Unreleased]: https://github.com/RazrFalcon/usvg/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/RazrFalcon/svgtypes/compare/v0.1.1...v0.2.0

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use svgdom;

/// Errors list.
#[derive(Fail, Debug)]
pub enum Error {
    /// Failed to find an SVG size.
    ///
    /// SVG size must be explicitly defined.
    /// Automatic image size determination is not supported.
    #[fail(display = "file doesn't have 'width', 'height' and 'viewBox' attributes. \
                      Automatic image size determination is not supported")]
    SizeDeterminationUnsupported,

    /// The `svg` node is missing.
    ///
    /// This error indicates an error in the preprocessor.
    #[fail(display = "the root svg node is missing")]
    MissingSvgNode,

    /// SVG size is not resolved.
    ///
    /// This error indicates an error in the preprocessor.
    #[fail(display = "invalid SVG size")]
    InvalidSize,

    /// `viewBox` attribute must be resolved.
    #[fail(display = "'viewBox' was not resolve'")]
    MissingViewBox,

    /// An invalid file extension.
    ///
    /// The extension should be 'svg' or 'svgz' in any case.
    #[fail(display = "invalid file extension")]
    InvalidFileExtension,

    /// SVG DOM errors.
    #[fail(display = "{}", _0)]
    SvgDom(svgdom::Error),

    /// IO errors.
    #[fail(display = "{}", _0)]
    Io(::std::io::Error),

    /// UTF-8 error.
    #[fail(display = "{}", _0)]
    UTF8(::std::string::FromUtf8Error),
}

impl From<svgdom::Error> for Error {
    fn from(value: svgdom::Error) -> Error {
        Error::SvgDom(value)
    }
}

impl From<::std::io::Error> for Error {
    fn from(value: ::std::io::Error) -> Error {
        Error::Io(value)
    }
}

impl From<::std::string::FromUtf8Error> for Error {
    fn from(value: ::std::string::FromUtf8Error) -> Error {
        Error::UTF8(value)
    }
}

/// A specialized `Result` type where the error is hard-wired to [`Error`].
///
/// [`Error`]: enum.Error.html
pub type Result<T> = ::std::result::Result<T, Error>;

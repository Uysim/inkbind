//! Extracting embedded raster images from a PDF page's resource
//! dictionary.
//!
//! Implemented as of Step 8. An embedded image is an `/XObject` resource
//! whose `/Subtype` is `/Image` (as opposed to `/Form`, a reusable content
//! stream, which this module ignores). `inkbind` decodes the two most
//! common cases without pulling in an image-decoding dependency:
//!
//! - **No filter** or **`/FlateDecode`/`/LZWDecode`** — the stream holds
//!   raw pixel samples once decompressed. Decompression reuses `lopdf`'s
//!   own tested [`Stream::decompressed_content`], which handles PNG
//!   predictors; it refuses on `/Subtype /Image` streams directly, so this
//!   module calls it on a clone with `/Subtype` removed.
//! - **`/DCTDecode`** (JPEG) — the stream bytes are already a complete,
//!   standard JPEG file. `inkbind` passes them through unchanged rather
//!   than decoding JPEG pixel data itself.
//!
//! Any other filter (or a multi-filter chain) is reported as
//! [`Error::Unsupported`] rather than silently dropped or guessed at.
//! `lopdf` is an implementation detail: its types never appear in this
//! module's public API.

use std::collections::BTreeSet;

use lopdf::{Dictionary, Object, ObjectId, Stream};

use crate::document::{Document, Page};
use crate::{Error, Result};

/// How [`Image::data`] is encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ImageFormat {
    /// `data` is raw, uncompressed pixel samples: `width * height` pixels,
    /// each `bits_per_component`-bit components in `color_space` order,
    /// row-major, with no row padding.
    Raw,
    /// `data` is a complete JPEG file, exactly as embedded in the PDF
    /// (`/Filter /DCTDecode`) — ready to write to disk or hand to any JPEG
    /// decoder.
    Jpeg,
}

/// A single embedded raster image, extracted from a page's `/XObject`
/// resources.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Image {
    /// The image's width in pixels, from the XObject's `/Width` entry.
    pub width: u32,
    /// The image's height in pixels, from the XObject's `/Height` entry.
    pub height: u32,
    /// Bits per color component, from the XObject's `/BitsPerComponent`
    /// entry. Defaults to 8 when absent, matching the PDF specification.
    pub bits_per_component: u8,
    /// The XObject's `/ColorSpace` name, when it is a direct name (e.g.
    /// `"DeviceRGB"`) or the first element of an array form (e.g.
    /// `"ICCBased"`, `"Indexed"`). `None` when `/ColorSpace` is absent or
    /// is itself an indirect reference (not dereferenced by this module).
    pub color_space: Option<String>,
    /// How `data` is encoded.
    pub format: ImageFormat,
    /// The image data itself; see [`ImageFormat`] for how to interpret it.
    pub data: Vec<u8>,
}

impl Document {
    /// Extracts every embedded raster image referenced by any page, in
    /// page order.
    pub fn images(&self) -> Result<Vec<Image>> {
        let mut images = Vec::new();
        for index in 0..self.page_count() {
            images.extend(self.page(index)?.images()?);
        }
        Ok(images)
    }
}

impl Page {
    /// Extracts the embedded raster images referenced by this page's
    /// resources (including resources inherited from ancestor page-tree
    /// nodes).
    pub fn images(&self) -> Result<Vec<Image>> {
        let Some(page_id) = self.page_id() else {
            return Ok(Vec::new());
        };
        let (resource_dict, resource_ids) = self.source.get_page_resources(page_id);

        let mut seen = BTreeSet::new();
        let mut images = Vec::new();
        if let Some(dict) = resource_dict {
            collect_images(dict, &self.source, &mut seen, &mut images)?;
        }
        for resource_id in resource_ids {
            if let Ok(dict) = self.source.get_dictionary(resource_id) {
                collect_images(dict, &self.source, &mut seen, &mut images)?;
            }
        }
        Ok(images)
    }
}

/// Reads `resources`' `/XObject` entries, keeps the ones whose `/Subtype`
/// is `/Image`, and decodes each into an [`Image`]. `seen` deduplicates
/// XObjects reachable through more than one resource dictionary (e.g. an
/// image shared via inherited resources).
fn collect_images(
    resources: &Dictionary,
    doc: &lopdf::Document,
    seen: &mut BTreeSet<ObjectId>,
    images: &mut Vec<Image>,
) -> Result<()> {
    let Ok(xobjects) = resources.get(b"XObject") else {
        return Ok(());
    };
    let xobject_dict = match xobjects {
        Object::Dictionary(dict) => Some(dict),
        Object::Reference(id) => doc.get_dictionary(*id).ok(),
        _ => None,
    };
    let Some(xobject_dict) = xobject_dict else {
        return Ok(());
    };

    for (_, value) in xobject_dict.iter() {
        let Ok(id) = value.as_reference() else {
            continue;
        };
        if !seen.insert(id) {
            continue;
        }
        let Ok(stream) = doc.get_object(id).and_then(Object::as_stream) else {
            continue;
        };
        if stream
            .dict
            .get(b"Subtype")
            .and_then(Object::as_name_str)
            .ok()
            != Some("Image")
        {
            continue;
        }
        images.push(decode_image(stream)?);
    }
    Ok(())
}

/// Decodes a single Image XObject stream into an [`Image`].
fn decode_image(stream: &Stream) -> Result<Image> {
    let dict = &stream.dict;
    let width = image_dimension(dict, b"Width")?;
    let height = image_dimension(dict, b"Height")?;
    let bits_per_component = dict
        .get(b"BitsPerComponent")
        .and_then(Object::as_i64)
        .unwrap_or(8) as u8;
    let color_space = color_space_name(dict);

    let filters = stream.filters().unwrap_or_default();
    let (format, data) = match filters.as_slice() {
        [] => (ImageFormat::Raw, stream.content.clone()),
        [name] if name == "DCTDecode" => (ImageFormat::Jpeg, stream.content.clone()),
        [name] if name == "FlateDecode" || name == "LZWDecode" => {
            (ImageFormat::Raw, decompress_image(stream)?)
        }
        _ => return Err(Error::Unsupported(format!("image filter {filters:?}"))),
    };

    Ok(Image {
        width,
        height,
        bits_per_component,
        color_space,
        format,
        data,
    })
}

/// Reads a required `/Width` or `/Height` integer entry from an Image
/// XObject dict.
fn image_dimension(dict: &Dictionary, key: &[u8]) -> Result<u32> {
    dict.get(key)
        .and_then(Object::as_i64)
        .map(|v| v as u32)
        .map_err(|_| {
            Error::InvalidPdf(format!(
                "image XObject missing /{}",
                String::from_utf8_lossy(key)
            ))
        })
}

/// Decompresses a `/FlateDecode` or `/LZWDecode` Image XObject stream by
/// reusing `lopdf`'s own [`Stream::decompressed_content`] on a clone with
/// `/Subtype` removed — `decompressed_content` otherwise refuses any
/// stream whose `/Subtype` is `/Image`, since general callers usually want
/// the raw, still-encoded bytes for such streams.
fn decompress_image(stream: &Stream) -> Result<Vec<u8>> {
    let mut clone = stream.clone();
    clone.dict.remove(b"Subtype");
    clone
        .decompressed_content()
        .map_err(|e| Error::InvalidPdf(format!("failed to decompress image stream: {e}")))
}

/// Reads an Image XObject's `/ColorSpace` entry as a name, when it is
/// directly available without dereferencing (see [`Image::color_space`]).
fn color_space_name(dict: &Dictionary) -> Option<String> {
    match dict.get(b"ColorSpace").ok()? {
        Object::Name(name) => Some(String::from_utf8_lossy(name).into_owned()),
        Object::Array(arr) => arr
            .first()
            .and_then(|o| o.as_name_str().ok())
            .map(str::to_owned),
        _ => None,
    }
}

//! Stamping text watermarks onto every page of a document.
//!
//! Implemented as of Step 12. [`Document::watermark_text`] draws `text` once
//! per page, using one of the 14 standard PDF fonts (`/Helvetica` — no font
//! embedding needed) and a `/ca` (non-stroking alpha) `/ExtGState` for real
//! transparency, then appends the drawing as an additional content stream
//! via `lopdf`'s own [`lopdf::Document::add_page_contents`] — every existing
//! content stream is left untouched.
//!
//! A page's resources are **materialized**: rather than risk shadowing
//! resources a page currently only inherits from an ancestor `/Pages` node
//! (which would break the page's own existing fonts/images if it were given
//! a fresh, watermark-only `/Resources` dict), this module first flattens
//! the page's effective resources (its own dict merged over anything
//! inherited via `/Parent`, the same walk [`crate::images`] uses) into a
//! single dict that becomes the page's own `/Resources`, then merges the
//! watermark's font and `/ExtGState` entries into that — without disturbing
//! sibling entries already in `/Font` or `/ExtGState`. Same flattening
//! spirit as [`crate::merge`], applied per-page rather than across whole
//! trees.
//!
//! Two simplifications, documented rather than engineered around (same
//! class as prior steps' inheritance/indirect-reference notes): the font
//! has no `/Encoding` entry, so `lopdf` reads and writes it via its default
//! `StandardEncoding` — only ASCII/Latin-range text round-trips correctly
//! through [`crate::Page::text`]; and the watermark's text is positioned
//! with its **baseline starting at the page's center point**, rotated by
//! [`WatermarkOptions::rotation`] degrees, rather than truly centered on its
//! own bounding box — standard-font glyph-width metrics aren't loaded, and
//! inkbind does not want an embedded-font dependency just for that. `lopdf`
//! is an implementation detail: its types never appear in this module's
//! public API.

use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Dictionary, Object, ObjectId};

use crate::document::{map_lopdf_error, Document};
use crate::{Error, Result};

const FONT_RESOURCE_NAME: &str = "InkbindWatermarkFont";
const GRAPHICS_STATE_RESOURCE_NAME: &str = "InkbindWatermarkGS";

/// Options controlling how [`Document::watermark_text`] draws its text.
///
/// Not `#[non_exhaustive]`: unlike [`crate::Error`] or [`crate::Image`],
/// this is a plain input struct meant to be built with
/// `..WatermarkOptions::default()`, which requires every field to be
/// publicly constructible.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WatermarkOptions {
    /// The font size, in points.
    pub font_size: f32,
    /// The counterclockwise rotation of the watermark text, in degrees,
    /// about the page's center point.
    pub rotation: f32,
    /// The watermark's non-stroking alpha (opacity), from `0.0`
    /// (fully transparent) to `1.0` (fully opaque).
    pub opacity: f32,
}

impl Default for WatermarkOptions {
    fn default() -> Self {
        WatermarkOptions {
            font_size: 48.0,
            rotation: 45.0,
            opacity: 0.3,
        }
    }
}

impl Document {
    /// Stamps `text` onto every page of the document, using `options` to
    /// control its size, rotation, and opacity.
    ///
    /// Returns [`Error::InvalidOpacity`] if `options.opacity` is outside
    /// `0.0..=1.0`.
    pub fn watermark_text(&self, text: &str, options: WatermarkOptions) -> Result<Document> {
        if !(0.0..=1.0).contains(&options.opacity) {
            return Err(Error::InvalidOpacity(options.opacity));
        }

        let mut result = self.inner().clone();

        let font_id = result.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        });
        let graphics_state_id = result.add_object(dictionary! {
            "Type" => "ExtGState",
            "ca" => options.opacity,
        });

        let page_ids: Vec<ObjectId> = result.get_pages().into_values().collect();
        for page_id in page_ids {
            let media_box = page_media_box(&result, page_id);
            let content = watermark_content(text, &options, media_box);
            let encoded = content.encode().map_err(map_lopdf_error)?;

            let mut resources = flattened_resources(&result, page_id);
            merge_sub_dict_entry(
                &mut resources,
                &result,
                b"Font",
                FONT_RESOURCE_NAME,
                font_id,
            );
            merge_sub_dict_entry(
                &mut resources,
                &result,
                b"ExtGState",
                GRAPHICS_STATE_RESOURCE_NAME,
                graphics_state_id,
            );
            result
                .get_dictionary_mut(page_id)
                .map_err(map_lopdf_error)?
                .set("Resources", resources);

            result
                .add_page_contents(page_id, encoded)
                .map_err(map_lopdf_error)?;
        }

        Ok(Document::from_inner(result))
    }
}

/// Builds the watermark's content stream: draw `text`, in the given
/// graphics state, with its baseline starting at `media_box`'s center,
/// rotated by `options.rotation` degrees counterclockwise.
fn watermark_content(
    text: &str,
    options: &WatermarkOptions,
    media_box: (f32, f32, f32, f32),
) -> Content<Vec<Operation>> {
    let (x0, y0, x1, y1) = media_box;
    let center_x = (x0 + x1) / 2.0;
    let center_y = (y0 + y1) / 2.0;
    let (sin, cos) = options.rotation.to_radians().sin_cos();

    Content {
        operations: vec![
            Operation::new("q", vec![]),
            Operation::new("gs", vec![GRAPHICS_STATE_RESOURCE_NAME.into()]),
            Operation::new("BT", vec![]),
            Operation::new(
                "Tf",
                vec![FONT_RESOURCE_NAME.into(), options.font_size.into()],
            ),
            Operation::new(
                "Tm",
                vec![
                    cos.into(),
                    sin.into(),
                    (-sin).into(),
                    cos.into(),
                    center_x.into(),
                    center_y.into(),
                ],
            ),
            Operation::new("Tj", vec![Object::string_literal(text)]),
            Operation::new("ET", vec![]),
            Operation::new("Q", vec![]),
        ],
    }
}

/// Reads `page_id`'s effective `/MediaBox`: its own entry if present,
/// otherwise the nearest ancestor's via `/Parent`, defaulting to US Letter
/// (`[0 0 612 792]`) if none is found anywhere.
fn page_media_box(doc: &lopdf::Document, page_id: ObjectId) -> (f32, f32, f32, f32) {
    let mut current = doc.get_dictionary(page_id).ok();
    while let Some(dict) = current {
        if let Ok(Object::Array(array)) = dict.get(b"MediaBox") {
            if let Some(rect) = parse_rect(array) {
                return rect;
            }
        }
        current = dict
            .get(b"Parent")
            .ok()
            .and_then(|object| object.as_reference().ok())
            .and_then(|id| doc.get_dictionary(id).ok());
    }
    (0.0, 0.0, 612.0, 792.0)
}

fn parse_rect(array: &[Object]) -> Option<(f32, f32, f32, f32)> {
    if let [a, b, c, d] = array {
        Some((
            a.as_float().ok()?,
            b.as_float().ok()?,
            c.as_float().ok()?,
            d.as_float().ok()?,
        ))
    } else {
        None
    }
}

/// Flattens `page_id`'s effective resources — its own inline `/Resources`
/// dict, if any, merged over whatever `doc.get_page_resources` finds by
/// walking `/Parent` — into a single, page-owned [`Dictionary`]. Top-level
/// keys nearer the page win over ones from an ancestor.
fn flattened_resources(doc: &lopdf::Document, page_id: ObjectId) -> Dictionary {
    let (own_resources, inherited_ids) = doc.get_page_resources(page_id);

    let mut merged = Dictionary::new();
    for &id in inherited_ids.iter().rev() {
        if let Ok(dict) = doc.get_dictionary(id) {
            for (key, value) in dict.iter() {
                merged.set(key.clone(), value.clone());
            }
        }
    }
    if let Some(dict) = own_resources {
        for (key, value) in dict.iter() {
            merged.set(key.clone(), value.clone());
        }
    }
    merged
}

/// Adds `entry_name` -> `entry_id` to `resources`' `key` sub-dictionary
/// (e.g. `/Font`, `/ExtGState`), preserving whatever entries that
/// sub-dictionary already had — whether it was inline or an indirect
/// reference into `doc`.
fn merge_sub_dict_entry(
    resources: &mut Dictionary,
    doc: &lopdf::Document,
    key: &[u8],
    entry_name: &str,
    entry_id: ObjectId,
) {
    let mut sub_dict = match resources.get(key) {
        Ok(Object::Dictionary(dict)) => dict.clone(),
        Ok(Object::Reference(id)) => doc.get_dictionary(*id).cloned().unwrap_or_default(),
        _ => Dictionary::new(),
    };
    sub_dict.set(entry_name, Object::Reference(entry_id));
    resources.set(key, sub_dict);
}

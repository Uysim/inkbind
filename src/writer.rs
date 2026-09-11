//! Building new PDF documents from scratch.
//!
//! Implemented as of Step 13. [`DocumentBuilder`] assembles a fresh
//! [`Document`] page by page: each [`PageSpec`] holds the page's size and
//! the [`TextLine`]s drawn on it, each placed at an explicit `(x, y)`
//! baseline position given by the caller — there is no automatic layout,
//! wrapping, or pagination. Every page shares one standard `/Type1
//! /BaseFont /Helvetica` font object (no embedding needed), the same
//! approach [`crate::watermark`] uses. Unlike every other module so far,
//! this one does not start from a cloned source document — it assembles a
//! `Pages`/`Catalog` tree from nothing, in the same hand-built style
//! [`crate::merge`] uses to reassemble one from several sources.
//!
//! Combined with [`Document::to_bytes`]/[`Document::save`] (added to
//! [`crate::document`] alongside this module), a document built this way
//! can be serialized and then reopened with
//! [`Document::from_bytes`]/[`Document::open`] like any other PDF — the
//! first inkbind step whose output is verified through an actual
//! serialize-then-reparse round trip, rather than only inspecting the
//! in-memory object graph.
//!
//! Same font-encoding caveat as [`crate::watermark`]: the font has no
//! `/Encoding` entry, so `lopdf` reads and writes it via its default
//! `StandardEncoding` — only ASCII/Latin-range text round-trips correctly
//! through [`crate::Page::text`]. `lopdf` is an implementation detail: its
//! types never appear in this module's public API.

use lopdf::content::{Content, Operation};
use lopdf::{dictionary, Dictionary, Object, Stream};

use crate::document::{map_lopdf_error, Document};
use crate::Result;

const FONT_RESOURCE_NAME: &str = "InkbindWriterFont";

/// US Letter page dimensions, in points (`width, height`): the default
/// [`PageSpec::size`].
pub const LETTER_SIZE: (f32, f32) = (612.0, 792.0);

/// A single line of text drawn on a generated page, at an explicit
/// position — there is no automatic layout or wrapping.
#[derive(Debug, Clone, PartialEq)]
pub struct TextLine {
    /// The text to draw. Only ASCII/Latin-range characters round-trip
    /// correctly; see this module's documentation.
    pub text: String,
    /// The x position of the text's baseline start, in points from the
    /// page's left edge.
    pub x: f32,
    /// The y position of the text's baseline, in points from the page's
    /// bottom edge.
    pub y: f32,
    /// The font size, in points.
    pub font_size: f32,
}

/// One page of a document under construction: its size and the text lines
/// drawn on it, in the order given.
#[derive(Debug, Clone, PartialEq)]
pub struct PageSpec {
    /// The page's `(width, height)`, in points. Defaults to
    /// [`LETTER_SIZE`].
    pub size: (f32, f32),
    /// The text lines to draw on the page, in order.
    pub lines: Vec<TextLine>,
}

impl Default for PageSpec {
    fn default() -> Self {
        PageSpec {
            size: LETTER_SIZE,
            lines: Vec::new(),
        }
    }
}

impl PageSpec {
    /// A page of `size` with no text lines yet.
    pub fn new(size: (f32, f32)) -> Self {
        PageSpec {
            size,
            lines: Vec::new(),
        }
    }

    /// Appends a text line to the page and returns `self`, for chaining.
    pub fn with_line(mut self, line: TextLine) -> Self {
        self.lines.push(line);
        self
    }
}

/// Builds a new [`Document`] from scratch, page by page.
///
/// ```
/// use inkbind::{DocumentBuilder, PageSpec, TextLine};
///
/// let mut builder = DocumentBuilder::new();
/// builder.add_page(PageSpec::default().with_line(TextLine {
///     text: "Hello, world!".to_owned(),
///     x: 72.0,
///     y: 720.0,
///     font_size: 24.0,
/// }));
///
/// let doc = builder.build()?;
/// assert_eq!(doc.page_count(), 1);
/// # Ok::<(), inkbind::Error>(())
/// ```
#[derive(Debug, Default)]
pub struct DocumentBuilder {
    pages: Vec<PageSpec>,
}

impl DocumentBuilder {
    /// A builder with no pages yet.
    pub fn new() -> Self {
        DocumentBuilder::default()
    }

    /// Appends a page and returns `&mut self`, for chaining.
    pub fn add_page(&mut self, page: PageSpec) -> &mut Self {
        self.pages.push(page);
        self
    }

    /// Assembles the added pages into a new [`Document`].
    ///
    /// Returns a valid, zero-page document if no pages were added.
    pub fn build(&self) -> Result<Document> {
        let mut doc = lopdf::Document::with_version("1.7");

        let font_id = doc.add_object(dictionary! {
            "Type" => "Font",
            "Subtype" => "Type1",
            "BaseFont" => "Helvetica",
        });

        let mut page_ids = Vec::with_capacity(self.pages.len());
        for page in &self.pages {
            let encoded = page_content(page).encode().map_err(map_lopdf_error)?;
            let content_id = doc.add_object(Stream::new(Dictionary::new(), encoded));

            let (width, height) = page.size;
            let page_id = doc.add_object(dictionary! {
                "Type" => "Page",
                "MediaBox" => vec![0.0.into(), 0.0.into(), width.into(), height.into()],
                "Resources" => dictionary! {
                    "Font" => dictionary! {
                        FONT_RESOURCE_NAME => font_id,
                    },
                },
                "Contents" => content_id,
            });
            page_ids.push(page_id);
        }

        let pages_id = doc.add_object(dictionary! {
            "Type" => "Pages",
            "Count" => page_ids.len() as i64,
            "Kids" => page_ids.iter().copied().map(Object::Reference).collect::<Vec<_>>(),
        });
        for &page_id in &page_ids {
            if let Ok(dict) = doc.get_dictionary_mut(page_id) {
                dict.set("Parent", pages_id);
            }
        }

        let catalog_id = doc.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        doc.trailer.set("Root", catalog_id);
        doc.max_id = doc.objects.keys().map(|(id, _)| *id).max().unwrap_or(0);

        Ok(Document::from_inner(doc))
    }
}

/// Builds a page's content stream: for each of its lines, in order, select
/// the shared font at the line's size and draw the text with its baseline
/// starting at `(line.x, line.y)`.
fn page_content(page: &PageSpec) -> Content<Vec<Operation>> {
    let mut operations = Vec::with_capacity(page.lines.len() * 5);
    for line in &page.lines {
        operations.push(Operation::new("BT", vec![]));
        operations.push(Operation::new(
            "Tf",
            vec![FONT_RESOURCE_NAME.into(), line.font_size.into()],
        ));
        operations.push(Operation::new("Td", vec![line.x.into(), line.y.into()]));
        operations.push(Operation::new(
            "Tj",
            vec![Object::string_literal(line.text.clone())],
        ));
        operations.push(Operation::new("ET", vec![]));
    }
    Content { operations }
}

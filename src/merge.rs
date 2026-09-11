//! Combining multiple PDF documents into one.
//!
//! Implemented as of Step 9, on top of the object-renumbering machinery
//! `lopdf` already provides (`Document::renumber_objects_with`) — this
//! module follows the pattern used by `lopdf`'s own `examples/merge.rs`,
//! minus its bookmark/outline handling, which `inkbind` does not support.
//! Pages from each input document are concatenated in order: the order
//! documents are passed to [`Document::merge`], then each document's own
//! page order. Every input's page tree is flattened to a single merged
//! `/Pages` node one level deep, so a page whose `/Resources` or
//! `/MediaBox` were inherited from an *intermediate* `/Pages` ancestor
//! (rather than being direct on the page, or inherited from its document's
//! own root `/Pages` node) loses that inheritance after merging — a
//! documented, non-blocking limitation, same class as prior steps'
//! `/Info`/`/ColorSpace` indirect-reference notes. `lopdf` is an
//! implementation detail: its types never appear in this module's public
//! API.

use lopdf::{Dictionary, Object, ObjectId};

use crate::document::Document;
use crate::Result;

impl Document {
    /// Combines `documents` into a single new [`Document`], concatenating
    /// their pages in order: the order `documents` are given, then each
    /// document's own page order.
    ///
    /// Returns a valid, zero-page document when `documents` is empty. The
    /// merged document's `/Info` metadata is always empty
    /// ([`Metadata::default`](crate::Metadata::default)) — there is no
    /// principled way to pick one input's metadata for the combined
    /// document, so none is propagated.
    pub fn merge(documents: &[Document]) -> Result<Document> {
        let mut merged = lopdf::Document::with_version("1.7");
        let mut next_id = 1u32;
        let mut page_ids: Vec<ObjectId> = Vec::new();
        let mut catalog: Option<(ObjectId, Dictionary)> = None;
        let mut pages_root_id: Option<ObjectId> = None;

        for document in documents {
            let mut source = document.inner().clone();
            source.renumber_objects_with(next_id);
            next_id = source.max_id + 1;

            page_ids.extend(source.get_pages().into_values());

            for (object_id, object) in source.objects {
                match object {
                    Object::Dictionary(dict) if dict.type_is(b"Catalog") => {
                        if catalog.is_none() {
                            catalog = Some((object_id, dict));
                        }
                    }
                    Object::Dictionary(dict) if dict.type_is(b"Pages") => {
                        if pages_root_id.is_none() {
                            pages_root_id = Some(object_id);
                        }
                        merged.objects.insert(object_id, Object::Dictionary(dict));
                    }
                    other => {
                        merged.objects.insert(object_id, other);
                    }
                }
            }
        }

        // Object IDs assigned above never went through `merged`'s own
        // `add_object`/`new_object_id`, so its counter needs to catch up
        // before either is used to mint the Pages/Catalog fallbacks below.
        merged.max_id = next_id.saturating_sub(1);

        let pages_root_id = pages_root_id.unwrap_or_else(|| merged.new_object_id());
        let mut pages_dict = match merged.objects.remove(&pages_root_id) {
            Some(Object::Dictionary(dict)) => dict,
            _ => Dictionary::new(),
        };
        pages_dict.set("Type", Object::Name(b"Pages".to_vec()));
        pages_dict.set("Count", page_ids.len() as i64);
        pages_dict.set(
            "Kids",
            page_ids
                .iter()
                .copied()
                .map(Object::Reference)
                .collect::<Vec<_>>(),
        );
        merged
            .objects
            .insert(pages_root_id, Object::Dictionary(pages_dict));

        for page_id in &page_ids {
            if let Some(Object::Dictionary(dict)) = merged.objects.get_mut(page_id) {
                dict.set("Parent", pages_root_id);
            }
        }

        let (catalog_id, mut catalog_dict) =
            catalog.unwrap_or_else(|| (merged.new_object_id(), Dictionary::new()));
        catalog_dict.set("Type", Object::Name(b"Catalog".to_vec()));
        catalog_dict.set("Pages", pages_root_id);
        catalog_dict.remove(b"Outlines");
        merged
            .objects
            .insert(catalog_id, Object::Dictionary(catalog_dict));
        merged.trailer.set("Root", catalog_id);

        merged.max_id = merged.objects.keys().map(|(id, _)| *id).max().unwrap_or(0);
        merged.renumber_objects();

        Ok(Document::from_inner(merged))
    }
}

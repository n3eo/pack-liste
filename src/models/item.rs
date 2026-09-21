use serde::{Deserialize, Serialize};

/// The per-item buffer applied on top of the rate-based amount:
/// a flat reserve added after it, or a multiplier scaling it —
/// never both (the exclusivity is a type guarantee, spec 010).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub(crate) enum QuantityBuffer {
    /// No buffer (the default)
    #[default]
    None,
    /// A flat amount added on top of the calculated amount,
    /// independent of trip length and multipliers
    Reserve(u32),
    /// A scaling factor applied to the rate-based amount
    Multiplier(f32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// One packing-list item: its name, category memberships, quantity,
/// and per-trip state (selected, packed, quantity mode).
pub(crate) struct Item {
    pub(crate) name: String,
    /// The categories this item belongs to; at least one. The first entry
    /// is the primary category (used for display grouping in trip files).
    pub(crate) categories: Vec<String>,
    pub(crate) quantity: u32,
    pub(crate) packed: bool,
    pub(crate) selected: bool,
    /// When false, the quantity is fixed and not recalculated from
    /// trip parameters (e.g. a toothbrush always stays at 1)
    pub(crate) auto_quantity: bool,
    /// How many of this item one trip day needs (spec 010);
    /// 1.0 = one per day, 0.15 ≈ one per week. Default 1.0.
    pub(crate) rate: f32,
    /// The buffer applied on top of the rate-based amount (spec 010):
    /// a flat reserve or a multiplier, never both. Default None.
    pub(crate) buffer: QuantityBuffer,
}

impl Item {
    /// Create an item: selected, unpacked, automatic quantity, with
    /// the default modifiers (rate 1.0, no buffer).
    pub(crate) fn new(name: String, categories: Vec<String>, quantity: u32) -> Self {
        Self {
            name,
            categories,
            quantity,
            packed: false,
            selected: true,
            auto_quantity: true,
            rate: 1.0,
            buffer: QuantityBuffer::None,
        }
    }

    /// The item's primary (first) category.
    pub(crate) fn primary_category(&self) -> &str {
        self.categories.first().map(String::as_str).unwrap_or("")
    }

    /// Whether the item belongs to the given category.
    pub(crate) fn in_category(&self, category: &str) -> bool {
        self.categories.iter().any(|c| c == category)
    }
}

/// Split a category string into its two display layers (spec 024):
/// the top layer (before the FIRST ": ") and the sub-category
/// (everything after it, verbatim — it may itself contain ": ").
/// A category without the separator has no sub-category: the whole
/// string is the top layer. Pure — the input is never rewritten; the
/// stored string remains the item's identity.
pub(crate) fn category_layers(category: &str) -> (&str, Option<&str>) {
    match category.split_once(": ") {
        Some((top, sub)) => (top, Some(sub)),
        None => (category, None),
    }
}

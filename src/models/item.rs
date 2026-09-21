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
/// One packing-list item: its name, labels, quantity, and per-trip
/// state (selected, packed, quantity mode). The labels are the item's
/// only grouping data (spec 025): flat plain strings, no hierarchy.
pub(crate) struct Item {
    pub(crate) name: String,
    /// The labels this item carries; one or more, order preserved as
    /// written. The item appears under every label it carries.
    pub(crate) labels: Vec<String>,
    /// When true, the item's selection and packed toggles are
    /// synchronized across all its labels (spec 025, US5): one
    /// selection state, one packed state. Meaningful only for
    /// multi-label items; default false.
    pub(crate) singleton: bool,
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
    /// Create an item: selected, unpacked, automatic quantity, a
    /// non-singleton, with the default modifiers (rate 1.0, no
    /// buffer).
    pub(crate) fn new(name: String, labels: Vec<String>, quantity: u32) -> Self {
        Self {
            name,
            labels,
            singleton: false,
            quantity,
            packed: false,
            selected: true,
            auto_quantity: true,
            rate: 1.0,
            buffer: QuantityBuffer::None,
        }
    }

    /// Whether the item carries the given label.
    pub(crate) fn has_label(&self, label: &str) -> bool {
        self.labels.iter().any(|l| l == label)
    }
}

# Feature Specification: Per-Item Quantity Configuration

**Feature Branch**: `010-item-quantity-config`

**Created**: 2026-09-20

**Status**: Draft

**Input**: User description:
"implement dynamic count calculation based on weather, activities, travel style and consider individual item config"

Refined 2026-09-20 with:
"Add item individual modifiers, for example I need 5 underwear for 5 days plus 2 reserve but one trouser is enough, on a 2 week travel 2 trousers would be appropriate with 3 underwear reserve"

> **Scope note**: The dynamic count calculation from trip parameters
> (weekdays, travel style, weather, activities) is already implemented and
> shipped (spec 005). This spec covers the missing piece in that
> description: the calculation must **consider individual item
> configuration**, so quantities are computed per item rather than applying
> one global multiplier uniformly to every item.
>
> The refinement adds two per-item concepts the original description
> implied but did not name: a **per-day rate** (underwear: one per day;
> trousers: about one per week) and a **flat reserve** added on top of the
> calculated amount (2 reserve underwear on a 5-day trip, 3 on a 14-day
> trip). Both are configured per item.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Per-Item Quantity Modifiers (Priority: P1)

The user configures an individual item with its own quantity modifiers: a **per-day rate** (how many of this item one day of the trip needs) and a **flat reserve** (a fixed extra amount added on top). The dynamic calculation applies the trip-parameter multipliers (weekdays, travel style, weather, activities) to each item's own rate, then adds the reserve. Different items get different quantities from the same trip parameters.

**Why this priority**: Today every automatically calculated item receives the identical quantity, which is wrong for most real lists. The user's own examples show why: underwear needs one per day plus a small reserve, while trousers need about one per week — a 5-day trip yields 7 underwear (5 + 2 reserve) but only 1 trouser; a 14-day trip yields 17 underwear (14 + 3 reserve) and 2 trousers. Per-item modifiers are the core value of "consider individual item config"; without them the calculation stays uniform.

**Independent Test**: Configure underwear with rate 1.0 and reserve 2, and trousers with rate 0.1 and reserve 0; calculate for a 5-day and a 14-day trip and verify the quantities match the expected values (7 and 1; 17 and 2 with the week-based rate).

**Acceptance Scenarios**:

1. **Given** underwear is configured with per-day rate 1.0 and reserve 2, **When** the list is calculated for a 5-day trip, **Then** the quantity is 7 (5 × 1.0, rounded, plus 2 reserve).
2. **Given** trousers are configured with a per-day rate that yields about one per week (e.g., 0.15), **When** the list is calculated for a 5-day trip, **Then** the quantity is 1 (0.75 rounds to 1, minimum 1); **and when** calculated for a 14-day trip, **Then** the quantity is 2 (2.1 rounds to 2).
3. **Given** underwear with rate 1.0 and reserve 3, **When** the list is calculated for a 14-day trip, **Then** the quantity is 17 (14 + 3 reserve).
4. **Given** the user changes the weather from "mixed" to "cold", **When** the list is recalculated, **Then** both items' quantities are scaled by the cold-weather multiplier before the reserve is added.

---

### User Story 2 - Configure the Per-Item Modifiers in the Item Editor (Priority: P1)

The user edits an item (existing add/edit item flow) and sets its per-day rate and reserve there. Both are part of the item's configuration, persisted with the item, and shown while editing.

**Why this priority**: The modifiers are only useful if the user can actually set them where items are already managed. Reusing the existing item editor keeps the interaction cost low.

**Independent Test**: Open the item editor for any item, change the rate and reserve, save, and verify the quantity recalculates immediately and both values survive an application restart.

**Acceptance Scenarios**:

1. **Given** the user opens the edit dialog for an item, **When** they change the per-day rate or reserve and save, **Then** the item's quantity recalculates immediately using the new values.
2. **Given** an item with non-default modifiers, **When** the application is restarted and the list reloaded, **Then** the rate and reserve are still configured on the item.
3. **Given** the user adds a new item, **When** no modifiers are provided, **Then** the item uses the defaults (rate 1.0, reserve 0) and behaves exactly as before this feature.

---

### User Story 3 - Fixed-Quantity Items Stay Untouched (Priority: P2)

The user marks an item as fixed-quantity (the existing auto/fixed toggle, e.g., a toothbrush that always stays at 1). The per-item modifiers do not apply to fixed items; their quantity never changes when trip parameters change.

**Why this priority**: The existing fixed/auto distinction (spec 005, EC-005) must keep working; per-item modifiers are an extension of the automatic path, not a replacement for the fixed path.

**Independent Test**: Mark an item fixed with quantity 1, change trip parameters and per-item modifiers, and verify the item's quantity stays 1.

**Acceptance Scenarios**:

1. **Given** an item configured as fixed with quantity 1, **When** any trip parameter or the item's own rate or reserve changes, **Then** the item's quantity remains 1.
2. **Given** the user switches an item from fixed to automatic, **When** the list is recalculated, **Then** the per-item modifiers apply from then on.

---

### User Story 4 - Backward-Compatible Configuration Files (Priority: P2)

The user opens a list or configuration written before this feature. Items without per-item modifiers are treated as having the defaults (rate 1.0, reserve 0), so old files load and calculate exactly as before.

**Why this priority**: The application has existing users with persisted lists and configs; a breaking format change would silently corrupt their data. Backward compatibility is required for the feature to be shippable.

**Independent Test**: Load a pre-feature configuration file and verify all items load, calculate with the default modifiers, and can be saved again without losing data.

**Acceptance Scenarios**:

1. **Given** a configuration file whose items carry no per-item modifiers, **When** it is loaded, **Then** every item is treated as rate 1.0 / reserve 0 and the calculated quantities match the pre-feature behavior.
2. **Given** a list saved by the new version, **When** it is opened by the calculation, **Then** items with explicit modifiers calculate with them and items without use the defaults.

---

### Edge Cases

- **EC-001**: What happens if a per-day rate is zero or negative?
  **Answer**: Rates are constrained to positive values; invalid input is rejected by the editor (minimum 0.05, two decimal places). The calculation never multiplies with a non-positive rate.
  **Acceptance Criteria**: No item can be saved with a rate of 0 or below; the editor rejects the input.

- **EC-002**: What happens if the calculated amount (before reserve) rounds to 0?
  **Answer**: The existing minimum-1 clamp (spec 005, FR-008) applies to the rate-based amount before the reserve is added, so every calculated quantity is at least 1 (rate-based) and at least 1 + reserve in total.
  **Acceptance Criteria**: For any valid rate, reserve, and trip parameters, the calculated quantity is at least 1.

- **EC-003**: What happens if the modifiers produce a very large quantity (e.g., rate 10 on a 30-day trip)?
  **Answer**: The existing cap rules apply: the calculation base is capped at 20 days (spec 005, FR-007); the final quantity (rate amount + reserve) is additionally capped at a sane maximum (99) so a misconfigured modifier cannot produce absurd numbers.
  **Acceptance Criteria**: No calculated quantity exceeds 99 for any valid input.

- **EC-004**: What happens to the per-item modifiers when an item is edited through the raw config editor?
  **Answer**: The rate and reserve are part of the item's persisted configuration and round-trip through save/load like the other item fields; the raw editor shows and preserves them.
  **Acceptance Criteria**: A save/load cycle preserves every item's rate and reserve exactly.

- **EC-005**: What happens when the user edits the quantity of an automatic item directly (existing behavior)?
  **Answer**: Directly editing an automatic item's quantity switches it to fixed (existing behavior); the per-item modifiers remain stored but are inactive while the item is fixed.
  **Acceptance Criteria**: Switching to fixed preserves the stored modifiers; switching back to automatic reactivates them.

- **EC-006**: Is the reserve scaled by trip parameters (weather, style, activities)?
  **Answer**: No. The reserve is a flat amount added after the rate-based amount is calculated and clamped; trip multipliers apply only to the rate-based part. This matches the user's examples (2 reserve on 5 days, 3 on 14 days — the reserve is chosen by the user per item, not derived).
  **Acceptance Criteria**: Changing weather/style/activities never changes the reserve contribution; changing the trip length never changes it either.

- **EC-007**: What is a sensible reserve range?
  **Answer**: 0 to 10, integer. A reserve is a small buffer by nature; larger needs are better expressed through the rate.
  **Acceptance Criteria**: The editor rejects reserves outside 0–10.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST calculate each automatic item's quantity from the item's own per-day rate, applying the trip-parameter multipliers (weekdays base, travel style, weather/activities), then adding the item's flat reserve.
- **FR-002**: System MUST provide defaults of per-day rate 1.0 and reserve 0 so items without explicit configuration calculate exactly as before this feature.
- **FR-003**: System MUST let the user view and change an item's per-day rate and reserve in the existing item add/edit dialog.
- **FR-004**: System MUST persist the per-day rate and reserve with the item in the configuration file and preserve them across save/load cycles.
- **FR-005**: System MUST NOT apply the per-item modifiers to fixed-quantity items; their quantity stays as configured.
- **FR-006**: System MUST reject per-day rates outside the valid range (0.05 to 10.0) and reserves outside 0 to 10 in the editor.
- **FR-007**: System MUST clamp every calculated quantity to a minimum of 1 and a maximum of 99.
- **FR-008**: System MUST load configuration files written before this feature without error, treating absent modifiers as the defaults (rate 1.0, reserve 0).
- **FR-009**: System MUST recalculate all automatic items' quantities immediately when a per-item rate or reserve changes.
- **FR-010**: System MUST apply trip-parameter multipliers only to the rate-based amount; the reserve is added flat, unaffected by trip parameters.

### Key Entities *(include if feature involves data)*

- **Item**: gains two per-item quantity modifiers alongside its existing auto/fixed flag: a per-day rate and a flat reserve.
- **Per-Day Rate**: a positive decimal (0.05–10.0, two decimal places) expressing how many of this item one trip day needs (1.0 = one per day, 0.15 ≈ one per week, 0.05 ≈ one per three weeks).
- **Reserve**: a flat whole number (0–10) added on top of the calculated amount, independent of trip length and multipliers (e.g., 2 spare underwear).
- **Quantity Calculation**: the existing deterministic rule (spec 005) extended per item: rate amount = base (weekdays, capped at 20) × travel-style multiplier × effective weather multiplier × per-day rate, rounded, clamped to at least 1; final quantity = rate amount + reserve, clamped to 1–99; fixed items unaffected.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Items with different per-item modifiers receive different quantities from identical trip parameters, per the calculation rule (e.g., 5-day trip: underwear rate 1.0 + reserve 2 → 7; trousers rate 0.15 + reserve 0 → 1).
- **SC-002**: 100% of pre-feature configuration files load and calculate identically to before (modifiers default to rate 1.0 / reserve 0).
- **SC-003**: Users can change an item's rate or reserve and see the recalculated quantity within 1 second.
- **SC-004**: Every calculated quantity stays within 1–99 for any valid modifiers and trip parameters.
- **SC-005**: All existing quantity-calculation behavior (fixed items, activity priority over weather, base cap, minimum 1) continues to pass its existing tests.
- **SC-006**: The reserve contribution to any item's quantity is invariant under changes to trip length, travel style, weather, and activities.

## Assumptions

- The dynamic calculation from weekdays, travel style, weather, and activities (spec 005) is the foundation and remains unchanged in its rule structure; this feature only adds the per-item dimension.
- Default modifiers (rate 1.0, reserve 0) reproduce the current behavior exactly, which is both the default and the backward-compatibility guarantee.
- The user's examples define the model: "5 underwear for 5 days plus 2 reserve" = rate 1.0 + reserve 2; "one trouser is enough [for 5 days], 2 trousers on a 2-week travel" = a rate of roughly one per week (≈0.15/day), expressible with two decimal places.
- The reserve is flat (not scaled by trip parameters) because the user's examples show it chosen per item and per trip preference, not derived from inputs.
- The rate range 0.05–10.0 and reserve range 0–10 cover realistic packing needs; values outside them are user error, not a supported use case.
- The per-item modifiers are edited in the existing item dialog rather than a new screen, to keep the UI surface unchanged.
- The persisted configuration format gains two optional fields per item; no other format changes.

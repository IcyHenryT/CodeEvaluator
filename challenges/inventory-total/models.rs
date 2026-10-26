#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryItem {
    pub name: String,
    pub unit_price_cents: u64,
    pub quantity: u64,
}

pub fn inventory_total(items: &[InventoryItem]) -> u64 {
    items
        .iter()
        .map(|item| item.unit_price_cents * item.quantity)
        .sum()
}

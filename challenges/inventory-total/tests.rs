use submission::{inventory_total, InventoryItem};

fn item(name: &str, unit_price_cents: u64, quantity: u64) -> InventoryItem {
    InventoryItem {
        name: name.to_owned(),
        unit_price_cents,
        quantity,
    }
}

#[test]
fn totals_multiple_inventory_lines() {
    let items = vec![item("notebook", 499, 3), item("pen", 125, 4)];

    assert_eq!(inventory_total(&items), 1_997);
}

#[test]
fn empty_inventory_has_zero_total() {
    assert_eq!(inventory_total(&[]), 0);
}

#[test]
fn zero_quantity_and_free_items_add_nothing() {
    let items = vec![
        item("out of stock", 9_999, 0),
        item("sample", 0, 100),
        item("book", 2_500, 1),
    ];

    assert_eq!(inventory_total(&items), 2_500);
}

#[test]
fn handles_large_exact_totals() {
    let items = vec![item("bulk", 1_000_000, 4_000), item("extra", 7, 9)];

    assert_eq!(inventory_total(&items), 4_000_000_063);
}

#[test]
fn does_not_modify_or_consume_inventory() {
    let items = vec![item("reusable", 250, 2)];

    assert_eq!(inventory_total(&items), 500);
    assert_eq!(items[0].name, "reusable");
    assert_eq!(inventory_total(&items), 500);
}

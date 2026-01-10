external printf(input: string, ...): int;

struct Item {
    id: int,
    name: string,
    count: int,
}

struct Inventory {
    items: array<Item, 16>,
    length: int,
}

# pub function empty_inventory(): Inventory {
#     Item i0 = Item { id: 0, name: "none", count: 0 };
#     Item i1 = Item { id: 0, name: "none", count: 0 };
#     Item i2 = Item { id: 0, name: "none", count: 0 };
#     Item i3 = Item { id: 0, name: "none", count: 0 };
#     Item i4 = Item { id: 0, name: "none", count: 0 };
#     Item i5 = Item { id: 0, name: "none", count: 0 };
#     Item i6 = Item { id: 0, name: "none", count: 0 };
#     Item i7 = Item { id: 0, name: "none", count: 0 };
#     Item i8 = Item { id: 0, name: "none", count: 0 };
#     Item i9 = Item { id: 0, name: "none", count: 0 };
#     Item i10 = Item { id: 0, name: "none", count: 0 };
#     Item i11 = Item { id: 0, name: "none", count: 0 };
#     Item i12 = Item { id: 0, name: "none", count: 0 };
#     Item i13 = Item { id: 0, name: "none", count: 0 };
#     Item i14 = Item { id: 0, name: "none", count: 0 };
#     Item i15 = Item { id: 0, name: "none", count: 0 };

#     array<Item, 16> arr = {
#         i0, i1, i2, i3,
#         i4, i5, i6, i7,
#         i8, i9, i10, i11,
#         i12, i13, i14, i15
#     };

#     Inventory inv = Inventory {
#         items: arr,
#         length: 0,
#     };

#     return inv;
# }

# pub function inv_count(inv: Inventory): int {
#     int s = 0;

#     if (inv.items[0].count > 0) { s = s + inv.items[0].count; }
#     if (inv.items[1].count > 0) { s = s + inv.items[1].count; }
#     if (inv.items[2].count > 0) { s = s + inv.items[2].count; }
#     if (inv.items[3].count > 0) { s = s + inv.items[3].count; }
#     if (inv.items[4].count > 0) { s = s + inv.items[4].count; }
#     if (inv.items[5].count > 0) { s = s + inv.items[5].count; }
#     if (inv.items[6].count > 0) { s = s + inv.items[6].count; }
#     if (inv.items[7].count > 0) { s = s + inv.items[7].count; }
#     if (inv.items[8].count > 0) { s = s + inv.items[8].count; }
#     if (inv.items[9].count > 0) { s = s + inv.items[9].count; }
#     if (inv.items[10].count > 0) { s = s + inv.items[10].count; }
#     if (inv.items[11].count > 0) { s = s + inv.items[11].count; }
#     if (inv.items[12].count > 0) { s = s + inv.items[12].count; }
#     if (inv.items[13].count > 0) { s = s + inv.items[13].count; }
#     if (inv.items[14].count > 0) { s = s + inv.items[14].count; }
#     if (inv.items[15].count > 0) { s = s + inv.items[15].count; }

#     return s;
# }

# pub function fill_inventory(): Inventory {
#     Inventory inv = empty_inventory();

#     Item a = Item { id: 1, name: "Sword", count: 1 };
#     Item b = Item { id: 2, name: "Shield", count: 1 };
#     Item c = Item { id: 3, name: "Potion", count: 3 };
#     Item d = Item { id: 4, name: "Gem", count: 5 };

#     inv.items[0] = a;
#     inv.items[1] = b;
#     inv.items[2] = c;
#     inv.items[3] = d;

#     inv.length = 4;

#     return inv;
# }

pub function main(): int {
    # Inventory inv = fill_inventory();
    
    Item i1 = Item { id: 7, name: "none", count: 6 };

    # int total_items = inv_count(inv);

    array<Item, 1> a = {i1};

    a[0].id = 934;

    printf("%i", a[0].id);

    return 0;
}
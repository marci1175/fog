external printf(msg: string, ...): void;

# Basic small structs -------------------------------------------------

struct Vec2 {
    x: int,
    y: int,
}

struct Color {
    r: int,
    g: int,
    b: int,
}

struct Person {
    age: int,
    name: string,
    height_cm: int,
    mood: string,
    day: string,
}

struct Item {
    id: int,
    name: string,
    count: int,
}

struct Inventory {
    items: array<Item, 16>,
    length: int,
}

struct Score {
    name: string,
    value: int,
}

struct Stats {
    min: int,
    max: int,
    sum: int,
    avg: int,
    grade: string,
}

struct TextLine {
    text: string,
    length: int,
}

struct TextDocument {
    lines: array<TextLine, 16>,
    count: int,
}

#-------------------------------------------------------------
# Math helpers
#-------------------------------------------------------------

pub function add(a: int, b: int): int {
    int r = a + b;
    return r;
}

pub function sub(a: int, b: int): int {
    int r = a - b;
    return r;
}

pub function mul(a: int, b: int): int {
    int r = a * b;
    return r;
}

pub function divi(a: int, b: int): int {
    int r = a / b;
    return r;
}

pub function clamp(value: int, lo: int, hi: int): int {
    int res = value;
    if (res < lo) {
        res = lo;
    }
    if (res > hi) {
        res = hi;
    }
    return res;
}

pub function abs_i(v: int): int {
    if (v < 0) {
        return 0 - v;
    }
    return v;
}

#-------------------------------------------------------------
# Vec2 helpers
#-------------------------------------------------------------

pub function make_vec2(x: int, y: int): Vec2 {
    Vec2 v = Vec2 { x: x, y: y };
    return v;
}

pub function add_vec2(a: Vec2, b: Vec2): Vec2 {
    Vec2 r = Vec2 { x: a.x + b.x, y: a.y + b.y };
    return r;
}

pub function dist2(v: Vec2): int {
    int xx = v.x * v.x;
    int yy = v.y * v.y;
    int s = xx + yy;
    return s;
}

#-------------------------------------------------------------
# Color helpers
#-------------------------------------------------------------

pub function make_color(r: int, g: int, b: int): Color {
    Color c = Color { r: r, g: g, b: b };
    return c;
}

pub function normalize_color(c: Color): Color {
    c.r = clamp(c.r, 0, 255);
    c.g = clamp(c.g, 0, 255);
    c.b = clamp(c.b, 0, 255);
    return c;
}

#-------------------------------------------------------------
# Inventory (no loops → huge unrolled logic)
#-------------------------------------------------------------

pub function empty_inventory(): Inventory {
    Item i0 = Item { id: 0, name: "none", count: 0 };
    Item i1 = Item { id: 0, name: "none", count: 0 };
    Item i2 = Item { id: 0, name: "none", count: 0 };
    Item i3 = Item { id: 0, name: "none", count: 0 };
    Item i4 = Item { id: 0, name: "none", count: 0 };
    Item i5 = Item { id: 0, name: "none", count: 0 };
    Item i6 = Item { id: 0, name: "none", count: 0 };
    Item i7 = Item { id: 0, name: "none", count: 0 };
    Item i8 = Item { id: 0, name: "none", count: 0 };
    Item i9 = Item { id: 0, name: "none", count: 0 };
    Item i10 = Item { id: 0, name: "none", count: 0 };
    Item i11 = Item { id: 0, name: "none", count: 0 };
    Item i12 = Item { id: 0, name: "none", count: 0 };
    Item i13 = Item { id: 0, name: "none", count: 0 };
    Item i14 = Item { id: 0, name: "none", count: 0 };
    Item i15 = Item { id: 0, name: "none", count: 0 };

    array<Item, 16> arr = {
        i0, i1, i2, i3,
        i4, i5, i6, i7,
        i8, i9, i10, i11,
        i12, i13, i14, i15
    };

    Inventory inv = Inventory {
        items: arr,
        length: 0,
    };

    return inv;
}

pub function fill_inventory(): Inventory {
    Inventory inv = empty_inventory();

    Item a = Item { id: 1, name: "Sword", count: 1 };
    Item b = Item { id: 2, name: "Shield", count: 1 };
    Item c = Item { id: 3, name: "Potion", count: 3 };
    Item d = Item { id: 4, name: "Gem", count: 5 };

    inv.items[0] = a;
    inv.items[1] = b;
    inv.items[2] = c;
    inv.items[3] = d;

    inv.length = 4;

    return inv;
}

pub function inv_count(inv: Inventory): int {
    int s = 0;

    if (inv.items[0].count > 0) { s = s + inv.items[0].count; }
    if (inv.items[1].count > 0) { s = s + inv.items[1].count; }
    if (inv.items[2].count > 0) { s = s + inv.items[2].count; }
    if (inv.items[3].count > 0) { s = s + inv.items[3].count; }
    if (inv.items[4].count > 0) { s = s + inv.items[4].count; }
    if (inv.items[5].count > 0) { s = s + inv.items[5].count; }
    if (inv.items[6].count > 0) { s = s + inv.items[6].count; }
    if (inv.items[7].count > 0) { s = s + inv.items[7].count; }
    if (inv.items[8].count > 0) { s = s + inv.items[8].count; }
    if (inv.items[9].count > 0) { s = s + inv.items[9].count; }
    if (inv.items[10].count > 0) { s = s + inv.items[10].count; }
    if (inv.items[11].count > 0) { s = s + inv.items[11].count; }
    if (inv.items[12].count > 0) { s = s + inv.items[12].count; }
    if (inv.items[13].count > 0) { s = s + inv.items[13].count; }
    if (inv.items[14].count > 0) { s = s + inv.items[14].count; }
    if (inv.items[15].count > 0) { s = s + inv.items[15].count; }

    return s;
}

pub function inv_slots(inv: Inventory): int {
    int n = 0;

    if (inv.items[0].count > 0) { n = n + 1; }
    if (inv.items[1].count > 0) { n = n + 1; }
    if (inv.items[2].count > 0) { n = n + 1; }
    if (inv.items[3].count > 0) { n = n + 1; }
    if (inv.items[4].count > 0) { n = n + 1; }
    if (inv.items[5].count > 0) { n = n + 1; }
    if (inv.items[6].count > 0) { n = n + 1; }
    if (inv.items[7].count > 0) { n = n + 1; }
    if (inv.items[8].count > 0) { n = n + 1; }
    if (inv.items[9].count > 0) { n = n + 1; }
    if (inv.items[10].count > 0) { n = n + 1; }
    if (inv.items[11].count > 0) { n = n + 1; }
    if (inv.items[12].count > 0) { n = n + 1; }
    if (inv.items[13].count > 0) { n = n + 1; }
    if (inv.items[14].count > 0) { n = n + 1; }
    if (inv.items[15].count > 0) { n = n + 1; }

    return n;
}

#-------------------------------------------------------------
# TextDocument builder
#-------------------------------------------------------------

pub function make_line(t: string): TextLine {
    TextLine l = TextLine {
        text: t,
        length: 0,
    };
    return l;
}

pub function make_doc(): TextDocument {
    TextLine l0 = make_line("Line 0");
    TextLine l1 = make_line("Line 1");
    TextLine l2 = make_line("Line 2");
    TextLine l3 = make_line("Line 3");
    TextLine l4 = make_line("Line 4");
    TextLine l5 = make_line("Line 5");
    TextLine l6 = make_line("Line 6");
    TextLine l7 = make_line("Line 7");
    TextLine l8 = make_line("Line 8");
    TextLine l9 = make_line("Line 9");
    TextLine l10 = make_line("Line 10");
    TextLine l11 = make_line("Line 11");
    TextLine l12 = make_line("Line 12");
    TextLine l13 = make_line("Line 13");
    TextLine l14 = make_line("Line 14");
    TextLine l15 = make_line("Line 15");

    array<TextLine,16> arr = {
        l0,l1,l2,l3,
        l4,l5,l6,l7,
        l8,l9,l10,l11,
        l12,l13,l14,l15
    };

    TextDocument doc = TextDocument {
        lines: arr,
        count: 16,
    };

    return doc;
}

#-------------------------------------------------------------
# Stats
#-------------------------------------------------------------

pub function make_scores(): array<Score, 8> {
    Score s0 = Score { name: "A", value: 10 };
    Score s1 = Score { name: "B", value: 25 };
    Score s2 = Score { name: "C", value: 15 };
    Score s3 = Score { name: "D", value: 5 };
    Score s4 = Score { name: "E", value: 40 };
    Score s5 = Score { name: "F", value: 20 };
    Score s6 = Score { name: "G", value: 50 };
    Score s7 = Score { name: "H", value: 12 };

    array<Score, 8> arr = { s0,s1,s2,s3,s4,s5,s6,s7 };
    return arr;
}

pub function stats_of_scores(): Stats {
    array<Score,8> s = make_scores();

    int v0 = s[0].value;
    int v1 = s[1].value;
    int v2 = s[2].value;
    int v3 = s[3].value;
    int v4 = s[4].value;
    int v5 = s[5].value;
    int v6 = s[6].value;
    int v7 = s[7].value;

    int total = v0 + v1 + v2 + v3 + v4 + v5 + v6 + v7;

    int min1 = v0;
    if (v1 < min1) { min1 = v1; }
    if (v2 < min1) { min1 = v2; }
    if (v3 < min1) { min1 = v3; }
    if (v4 < min1) { min1 = v4; }
    if (v5 < min1) { min1 = v5; }
    if (v6 < min1) { min1 = v6; }
    if (v7 < min1) { min1 = v7; }

    int max1 = v0;
    if (v1 > max1) { max1 = v1; }
    if (v2 > max1) { max1 = v2; }
    if (v3 > max1) { max1 = v3; }
    if (v4 > max1) { max1 = v4; }
    if (v5 > max1) { max1 = v5; }
    if (v6 > max1) { max1 = v6; }
    if (v7 > max1) { max1 = v7; }

    int count = 8;
    int avg = total / count;

    string grade = "Fail";
    if (avg >= 20) {
        grade = "Pass";
    }
    if (avg >= 30) {
        grade = "Good";
    }
    if (avg >= 40) {
        grade = "Excellent";
    }

    Stats st = Stats {
        min: min1,
        max: max1,
        sum: total,
        avg: avg,
        grade: grade,
    };

    return st;
}

#-------------------------------------------------------------
# Person helpers (no loops)
#-------------------------------------------------------------

pub function make_person(age: int, name: string, height: int, mood: string, day: string): Person {
    Person p = Person {
        age: age,
        name: name,
        height_cm: height,
        mood: mood,
        day: day,
    };
    return p;
}

pub function set_mood(p: Person, m: string): Person {
    p.mood = m;
    return p;
}

pub function set_day_str(p: Person, d: string): Person {
    p.day = d;
    return p;
}

#-------------------------------------------------------------
# MAIN
#-------------------------------------------------------------

pub function main(): int {
    printf("Fog big file start\n");

    # People ---------------------------------------------------
    Person p0 = make_person(18, "Alice", 170, "Neutral", "Monday");
    Person p1 = make_person(25, "Bob", 182, "Happy", "Friday");
    Person p2 = make_person(31, "Charlie", 165, "Sad", "Wednesday");

    p0 = set_mood(p0, "Happy");
    p1 = set_mood(p1, "Sad");
    p2 = set_day_str(p2, "Sunday");

    printf("P0 mood: %s\n", p0.mood);
    printf("P1 mood: %s\n", p1.mood);
    printf("P2 day: %s\n", p2.day);

    # Inventory ------------------------------------------------
    Inventory inv = fill_inventory();
    int total_items = inv_count(inv);
    int used = inv_slots(inv);

    printf("Inventory total count: %i\n", total_items);
    printf("Inventory used slots: %i\n", used);

    # Document -------------------------------------------------
    TextDocument doc = make_doc();
    printf("Doc first line: %s\n", doc.lines[0].text);

    # Stats ----------------------------------------------------
    Stats st = stats_of_scores();
    printf("Stats sum: %i\n", st.sum);
    printf("Stats avg: %i\n", st.avg);
    printf("Stats grade: %s\n", st.grade);

    # Vector math ----------------------------------------------
    Vec2 va = make_vec2(10, 20);
    Vec2 vb = make_vec2(3, -7);
    Vec2 vc = add_vec2(va, vb);

    int d = dist2(vc);
    printf("Vec2 dist2: %i\n", d);

    printf("Fog big file end\n");
    return 0;
}
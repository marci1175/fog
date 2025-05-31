struct a1 {
    a: int,
}

struct c1 {
    a: int,
}

struct b1 {
    a: int,
    sup: a1,
}

function main(): int {
    a1 hello = a1 {
        a: 92,
    };

    b1 hello = b1 {
        a: 92,
        sup: c1 {a: 2,},
    };

    return 0;
}
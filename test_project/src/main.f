import printf(str: string, val: int, val2: float, val3: string): int;

struct kg {
    inner: float,
}

struct marci {
    suly: kg,
}

function main(): int {
    marci szemely = marci { suly: kg { inner: 72.3 } };
    
    printf("value: %d, %f, %s", szemely.suly.inner as int, szemely.suly.inner, "szia");

    return 0;
}
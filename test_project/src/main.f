import printf(str: string, val: int, val2: float): int;

struct kg {
    inner: float,
}

struct marci {
    suly: kg,
}

function main(): int {
    marci szemely = marci { suly: kg { inner: 72.3 } };
    
    printf("value: %d, %f", szemely.suly.inner as int, szemely.suly.inner);

    return 0;
}
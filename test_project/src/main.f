external printf(a: string, ...): int;
external struct_ptr_test(): ptr;
external tfon_from_pointer(p: ptr): int;

struct telefon {
    szam: int,
    ido: int,
}

pub function foo(): ptr {
    return struct_ptr_test();
}

pub function main(): int {
    ptr a = foo();

    printf("%p\n", a);

    int szam = tfon_from_pointer(a);

    printf("Szam: %i", szam);

    return 0;
}
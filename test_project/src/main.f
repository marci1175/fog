struct telefon {
    szam: int,
    ido: int,
}

external printf(a: string, ...): int;
external struct_ptr_test(): ptr<telefon>;
external tfon_from_pointer(p: ptr): int;

pub function foo(): ptr<telefon> {
    return struct_ptr_test();
}

pub function main(): int {
    ptr<telefon> a = ref struct_ptr_test();
    telefon b = deref a;
    return b.szam;
}
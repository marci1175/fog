pub function main(): int {
    array<int, 4> marci = {1, 2, 3, 4};
    ptr egy_ptr = ref marci[0];
    int egy = deref egy_ptr;
    int hallo = (-2);
    return egy;
}

pub function marci(): string {
    return "Marci";
}

@feature "asdd"
pub function wello(lhs: int, rhs: int): int {
    return lhs * rhs - lhs;
}
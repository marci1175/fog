function sum(lhs: int, rhs: int): int {
    return lhs + rhs;
}

function main(): int {
    int a = 3 + (sum(lhs = 2, rhs = 23) - 2 / 2 * (23 - 2));
}
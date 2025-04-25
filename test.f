function sum(lhs: int, rhs: int): int {
    return lhs + rhs;
}

function main(): int {
    int b = sum(lhs = sum(lhs = 31 - 2, rhs = 2), rhs = 3);
}
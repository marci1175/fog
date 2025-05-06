function sum(lhs: int, rhs: float): int {
    return (lhs + rhs as int);
}

function main(): int {
    int a;

    a = test();
    
    a = sum(lhs = 2, rhs = 2.2);

    return 0;
}

function test(): int {
    return 0;
}
external function printf(str: string, ...): int;

public function main(): int {
    const int x = 2;

    return 0;
}

@feature "testing"
public function wello(lhs: int, rhs: int): int {
    return rhs * (rhs - rhs * (rhs - rhs * (rhs - rhs * (rhs - rhs * (rhs - rhs * (rhs - rhs * (rhs - rhs * (rhs - rhs * (rhs - rhs * (rhs - rhs * (rhs - 2)))))))))));
}
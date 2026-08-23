import marci::asd::hello as marci;
import "helper.f";
import helper::functions::add as marci;

pub function main(): int {
    array<int, 4> numbers = {1, 2, 3, 4};
    
    ptr item_ptr = ref numbers[0];
    # ptr<array<int, 4>> array_ptr = ref numbers;

    string name = "Alma";
    int number = wello(10, 24);

    println("Name: %s, Number: %i", name, number);

    return (deref item_ptr) - 1;
}

@feature "testing"
pub function wello(lhs: int, rhs: int): int {
    return lhs * (rhs - lhs);
}
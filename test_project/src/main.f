external function print(str: string, ...): int;
external static int counter;

import marci::asd::hello as marci;
import "helper.f";
import helper::functions::add;

pub function main(): int {
    var array<int, 4> numbers = {1, 2, 3, 4};
    
    const ptr item_ptr = ref numbers[0];
    const ptr<array<int, 4>> array_ptr = ref numbers;

    const string name = "Alma";
    const int number = wello(10, 24);

    println("Name: %s, Number: %i", name, number);

    return (deref item_ptr) - 1;
}

@feature "testing"
pub function wello(lhs: int, rhs: int): int {
    return lhs * (rhs - lhs);
}
import "masikmain.f";

import dep1::printn;
import masikmain::marci;

external printf(a: string, ...): int;

pub function main(): int {
    printn(marci());

    return 0;
}
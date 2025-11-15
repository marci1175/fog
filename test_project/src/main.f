import "masikmain.f";

import dep1::hi_from_ffi;
import masikmain::marci;

external printf(a: string, ...): int;

pub function main(): int {
    hi_from_ffi();

    return 0;
}
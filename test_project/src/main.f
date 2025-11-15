import dep1::hi_from_ffi;
import dep1::make_alma;
import dep1::printn;

struct Alma {
    szam: int,
    masik_szam: int,
}

external printf(a: string, ...): int;

pub function main(): int {
    Alma random_alma = make_alma();
    
    printn(random_alma.szam);
    
    return 0;
}
import dep1::kedvencek::hello::kedvenc;
external printf(a: string, ...): int;

pub function main(): int {
    kedvenc();
    
    return 0;
}
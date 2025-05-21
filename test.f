import print(str: string): int;

function main(): int {
    string test = "123456789\0";
    
    print(str = test);
    print(str = test);
    print(str = test);

    return 0;
}
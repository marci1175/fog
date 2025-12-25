# import dependency-test::szia;
# pub function main(): int {
#     szia();
    
#     return 0;
# }

# struct a {
#     field1: array<int, 5>,
#     field2: array<int, 5>,
#     field3: array<int, 5>,
#     field4: array<int, 5>,
# }

# pub function main(): int {  
#     a b = a { field1: {0, 1, 2, 3, 4}, field2: {0, 1, 2, 3, 4}, field3: {0, 1, 2, 3, 4}, field4: {0, 1, 2, 3, 4} };

#     a.field1[2] = 400;

#     printf("%i", a.field1[2]);

#     return 0;
# }

external printf(inp: string, ...): int;

enum<string> allat {
    kutya = "kutya",
    cica = "cica",
}

pub function fn(n: allat): allat {
    return n;
}

pub function main(): int {
    allat macska = allat::cica;

    allat kutya = fn(allat::kutya);

    printf("%s, %s", macska, kutya);

    return 0;
}
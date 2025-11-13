external printf(input: string, ...): int;

@feature "alma"
libpub function kedvenc(): void {
    printf("Alma");
}

@feature "marci"
libpub function kedvenc(): void {
    printf("Marci");
}

libpub function printn(x: int): int {
    return printf("%i", x);
}
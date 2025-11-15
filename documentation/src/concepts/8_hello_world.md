# Hello World!

## Writing the code for the project

Fog source files always end with the `.f` extension. A wide range of naming schemes can be used with fog support them all.

**Warning⚠️: File names cannot contain any special characters except "_".**

Navigate to: `%project-name%/src/main.f`

And enter the code:

```fog
external puts(msg: string): int;

function main(): int {
    puts("Hello World!");
    
    return 0;
}
```

The compiler can output LLVM-IR or even link automaticly, with clang (This requires clang to be added to `$PATH`).

When we automaticly want to run code we have written we need to run `fog run`. This compiles the code and links it with the built in project linker (Which is a wrapper around clang).

To only output LLVM-IR `fog compile` should be called.

> All build arctifacts are placed in the `build_path` defined in the project configuration.

**Example code of running the code above:**

```console
$ fog run
<compiler output>
Running `<path to the linked binary>`
Hello World!
```

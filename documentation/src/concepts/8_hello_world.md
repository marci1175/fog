# Hello World!

## Writing the code for the project

Fog source files always end with the `.f` extension. A wide range of naming schemes can be used with fog support them all.

**Warning⚠️: File names cannot contain any special characters except "_".**

Navigate to: `%project-name%/src/main.f`

And enter the code:

```fog
import puts(msg: string): int;

function main(): int {
    puts("Hello World!");
    
    return 0;
}
```

Save the file, run the command `fog c` and navigate to `./output`.

Here, you will see your binary's LLVM-IR which can be then parsed by a linker to produce a valid binary.
Use your preferred method of linking this file, to create an exe.

My preferred way of linking is via [Clang](https://clang.llvm.org/), so I am going to use that.

```console
$ fog c
$ clang %project-name%.ll
$ ./%project-name%.exe
Hello World!
```

If you can see "Hello World!" in your console, congratulations! You have created your first ever application with Fog.

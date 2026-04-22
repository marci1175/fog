# Compiler instructions / hints

Compiler instructions (otherwise called as compiler hints) can be occasionally useful when handling specific code.
In the language a compiler instructions can be present before any item (struct, function, etc) and will modify that items specific behavior accordingly.

> Please note that not all items may support compiler instructions in the latest release of the compiler.

**Most instructions directly translate to llvm instructions.**

## Instructions list

|Instruction|Behavior|
|---------------|-----------|
|Feature|Can be used to limit a function visibility to a specific feature when contained in a dependency.|
|Cold|Indicates to the compiler that the function is not used frequently.|
|NoFree|Indicates that the function will not call any memory freeing operation.|
|Inline|Inlines the function wherever it is used.|
|NoUnWind|*(Unused)*|

# Linking and the building process

When compiling the compiler first builds all the different dependencies a project could have. The compiler puts all the build arctifacts into the specified build folder. It then stores the path of the compiled LLVM-IR of the dependency.

Additional linking materal can be provided, for example when wanting to use external function from another pre-compiled library. (Using FFI)

The information related to the building of the project is then output into a build manifest file.

**Example of a build manifest file:**

```.
build_output_paths = ['C:\Users\marci\Desktop\fog\test_project\deps\dep1\out\dep1.ll', 'C:\Users\marci\Desktop\fog\test_project\out\test_project.ll']
additional_linking_material = []
output_path = 'C:\Users\marci\Desktop\fog\test_project\out\test_project.exe'
```

This build manifest file can only be read by the proprietary fog linker, which is a wrapper around clang with a parser.

> The LLVM-IR files can be linked manually with any linker, however it is far less intuitive compared to the builtin linker.

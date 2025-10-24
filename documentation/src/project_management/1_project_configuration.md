# Project Configuration

To compile or work on a project, every project is required to have its own configuration file. This configuration file provides crucial information to the compiler, LSP, and the user alike.

**The default project configuration looks something like this:**

```toml
name = "test_project"
is_library = false
version = "0.0.1"
build_path = "out"
additional_linking_material = []

[dependencies]
```

| Field Name                  | Usage                                                                                                        |
| --------------------------- | ------------------------------------------------------------------------------------------------------------ |
| name                        | Used for naming your project. It is also used to specify the library when importing a function.              |
| is_library                  | Tells the compiler not to search for a main function and enables the project to be imported as a dependency. |
| version                     | Specifies the version of a project. This is used to identify multiple editions of the same dependency.       |
| build_path                  | Tells the compiler where to place the build artifacts.                                                       |
| additional_linking_material | Tells the linker which additional files to link the object files with.                                       |
| dependencies                | Specifies the dependencies the project uses.                                                                 |

**Config file composition:**

```toml
name = <string>
is_library = <bool>
version = <version> # This must follow the semver specification.
build_path = <path>
additional_linking_material = [<path>, <path>, ...]

[dependencies]
<dependency name> = { version = <version>, features = [<feature name>, <feature name>, ...] }
<dependency name> = { version = <version>, features = [<feature name>, <feature name>, ...] }
...
```

> Learn more about [SemVer here](https://semver.org/).

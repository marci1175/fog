/// Handles the llvm-ir generation od debug symbols and information.
pub mod debug;
pub mod intrinsics;
/// Handles all the custom items inside the language. (Functions, structs, enums, traits, etc.)
pub mod items;
/// Memory management in the programming language
pub mod mem;
/// Inializes llvm and generates the llvm-ir from ASTs.
pub mod startup;

/// Handles the llvm-ir generation od debug symbols and information.
pub mod debug;
/// Inializes llvm and generates the llvm-ir from ASTs.
pub mod startup;
/// Handles all the custom items inside the language. (Functions, structs, enums, traits, etc.)
pub mod items;
/// Memory management in the programming language
pub mod mem;
pub mod intrinsics;
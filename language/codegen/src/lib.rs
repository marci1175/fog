/// Handles the llvm-ir generation od debug symbols and information.
pub mod debug;
/// Generates the llvm-ir from ASTs.
pub mod irgen;
/// Memory management in the programming language
pub mod mem;
/// Handles all the custom items inside the language. (Functions, structs, enums, traits, etc.) 
pub mod items;
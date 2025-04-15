pub mod type_system;

#[derive(Debug, strum_macros::EnumDiscriminants)]
pub enum Type {
    I32(i32),
    F32(f32),
    U32(u32),
    U8(u8),

    String(String),
    Boolean(bool),

    Void,
}

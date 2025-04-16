pub mod type_system;

#[derive(Debug, strum_macros::EnumDiscriminants, Clone, PartialEq)]
pub enum Type {
    I32(i32),
    F32(f32),
    U32(u32),
    U8(u8),

    String(String),
    Boolean(bool),

    Void,
}

impl From<TypeDiscriminants> for Type {
    fn from(value: TypeDiscriminants) -> Self {
        match value {
            TypeDiscriminants::I32 => Self::I32(0),
            TypeDiscriminants::F32 => Self::F32(0.0),
            TypeDiscriminants::U32 => Self::U32(0),
            TypeDiscriminants::U8 => Self::U8(0),
            TypeDiscriminants::String => Self::String(String::new()),
            TypeDiscriminants::Boolean => Self::Boolean(false),
            TypeDiscriminants::Void => Self::Void,
        }
    }
}
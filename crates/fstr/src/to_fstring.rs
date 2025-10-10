use crate::FString;

pub trait ToFString {
    fn to_fstring(&self) -> FString;
}

impl ToFString for &str {
    /// this WILL allocate even on static strings.
    fn to_fstring(&self) -> FString {
        FString::new(self)
    }
}
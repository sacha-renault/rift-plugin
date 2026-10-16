#[derive(Debug, Default, Clone, Copy)]
pub struct Tags(pub u32);

impl From<u32> for Tags {
    fn from(v: u32) -> Tags {
        Tags(v)
    }
}

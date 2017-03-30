use std::io;

mod module;
mod section;
mod primitives;
mod types;
mod import_entry;

pub use self::module::Module;
pub use self::section::Section;
pub use self::import_entry::ImportEntry;
pub use self::primitives::{VarUint32, VarUint7, VarUint1, VarInt7, Uint32, CountedList};
pub use self::types::ValueType;

pub trait Deserialize : Sized {
    type Error;
    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error>;
}

#[derive(Debug)]
pub enum Error {
    UnexpectedEof,
    InconsistentLength { expected: usize, actual: usize },
    Other(&'static str),
    HeapOther(String),
    UnknownValueType(i8),
    NonUtf8String,
    UnknownExternalKind(u8),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::HeapOther(format!("I/O Error: {}", err))
    }
}

struct Unparsed(pub Vec<u8>);

impl Deserialize for Unparsed {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let len = VarUint32::deserialize(reader)?.into();
        let mut vec = vec![0u8; len];
        reader.read_exact(&mut vec[..])?;
        Ok(Unparsed(vec))
    }
}

impl From<Unparsed> for Vec<u8> {
    fn from(u: Unparsed) -> Vec<u8> {
        u.0
    }
}

pub fn deserialize_file<P: AsRef<::std::path::Path>>(p: P) -> Result<Module, Error> {
    use std::io::Read;

    let mut contents = Vec::new();
    ::std::fs::File::open(p)?.read_to_end(&mut contents)?;

    deserialize_buffer(contents)
}

pub fn deserialize_buffer<T: Deserialize>(contents: Vec<u8>) -> Result<T, T::Error> {
    let mut reader = io::Cursor::new(contents);
    T::deserialize(&mut reader)
}
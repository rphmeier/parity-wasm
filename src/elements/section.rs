use std::io;
use super::{
    Serialize,
    Deserialize,
    Unparsed,
    Error,
    VarUint7,
    VarUint32,
    CountedList,
    ImportEntry,
    MemoryType,
    TableType,
    ExportEntry,
    GlobalEntry,
    Func,
    FuncBody,
    ElementSegment,
    DataSegment,
    CountedWriter,
    CountedListWriter,
};

use super::types::Type;

pub enum Section {
    Unparsed {
        id: u8,
        payload: Vec<u8>,
    },
    Custom(Vec<u8>),
    Type(TypeSection),
    Import(ImportSection),
    Function(FunctionsSection),
    Table(TableSection),
    Memory(MemorySection),
    Global(GlobalSection),
    Export(ExportSection),
    Start(u32),
    Element(ElementSection),
    Code(CodeSection),
    Data(DataSection),
}

impl Deserialize for Section {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        let id = match VarUint7::deserialize(reader) {
            // todo: be more selective detecting no more section
            Err(_) => { return Err(Error::UnexpectedEof); },
            Ok(id) => id,
        };

        Ok(
            match id.into() {
                0 => {
                    Section::Custom(Unparsed::deserialize(reader)?.into())
                },
                1 => {
                    Section::Type(TypeSection::deserialize(reader)?)
                },
                2 => {
                    Section::Import(ImportSection::deserialize(reader)?)
                },
                3 => {
                    Section::Function(FunctionsSection::deserialize(reader)?)
                },
                4 => {
                    Section::Table(TableSection::deserialize(reader)?)
                },
                5 => {
                    Section::Memory(MemorySection::deserialize(reader)?)
                },
                6 => {
                    Section::Global(GlobalSection::deserialize(reader)?)
                },
                7 => {
                    Section::Export(ExportSection::deserialize(reader)?)
                },
                8 => {
                    let _section_length = VarUint32::deserialize(reader)?;
                    Section::Start(VarUint32::deserialize(reader)?.into())
                },
                9 => {
                    Section::Element(ElementSection::deserialize(reader)?)
                },
                10 => {
                    Section::Code(CodeSection::deserialize(reader)?)
                },
                11 => {
                    Section::Data(DataSection::deserialize(reader)?)
                },
                _ => {
                    Section::Unparsed { id: id.into(), payload: Unparsed::deserialize(reader)?.into() }
                }
            }
        )
    }    
}

pub struct TypeSection(Vec<Type>);

impl TypeSection {
    pub fn types(&self) -> &[Type] {
        &self.0
    }
}

impl Deserialize for TypeSection {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        // todo: maybe use reader.take(section_length)
        let _section_length = VarUint32::deserialize(reader)?;
        let types: Vec<Type> = CountedList::deserialize(reader)?.into_inner();
        Ok(TypeSection(types))
    }   
}

pub struct ImportSection(Vec<ImportEntry>);

impl ImportSection {
    pub fn entries(&self) -> &[ImportEntry] {
        &self.0
    }
}

impl Deserialize for ImportSection {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        // todo: maybe use reader.take(section_length)
        let _section_length = VarUint32::deserialize(reader)?;
        let entries: Vec<ImportEntry> = CountedList::deserialize(reader)?.into_inner();
        Ok(ImportSection(entries))
    }   
}

pub struct FunctionsSection(Vec<Func>);

impl FunctionsSection {
    pub fn entries(&self) -> &[Func] {
        &self.0
    }
}

impl Deserialize for FunctionsSection {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        // todo: maybe use reader.take(section_length)
        let _section_length = VarUint32::deserialize(reader)?;
        let funcs: Vec<Func> = CountedList::<VarUint32>::deserialize(reader)?
            .into_inner()
            .into_iter()
            .map(|f| Func::new(f.into()))
            .collect();
        Ok(FunctionsSection(funcs))
    }
}

pub struct TableSection(Vec<TableType>);

impl TableSection {
    pub fn entries(&self) -> &[TableType] {
        &self.0
    }
}

impl Deserialize for TableSection {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        // todo: maybe use reader.take(section_length)
        let _section_length = VarUint32::deserialize(reader)?;
        let entries: Vec<TableType> = CountedList::deserialize(reader)?.into_inner();
        Ok(TableSection(entries))
    }   
}

pub struct MemorySection(Vec<MemoryType>);

impl MemorySection {
    pub fn entries(&self) -> &[MemoryType] {
        &self.0
    }
}

impl Deserialize for MemorySection {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        // todo: maybe use reader.take(section_length)
        let _section_length = VarUint32::deserialize(reader)?;
        let entries: Vec<MemoryType> = CountedList::deserialize(reader)?.into_inner();
        Ok(MemorySection(entries))
    }   
}

pub struct GlobalSection(Vec<GlobalEntry>);

impl GlobalSection {
    pub fn entries(&self) -> &[GlobalEntry] {
        &self.0
    }
}

impl Deserialize for GlobalSection {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        // todo: maybe use reader.take(section_length)
        let _section_length = VarUint32::deserialize(reader)?;
        let entries: Vec<GlobalEntry> = CountedList::deserialize(reader)?.into_inner();
        Ok(GlobalSection(entries))
    }   
}

pub struct ExportSection(Vec<ExportEntry>);

impl ExportSection {
    pub fn entries(&self) -> &[ExportEntry] {
        &self.0
    }
}

impl Deserialize for ExportSection {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        // todo: maybe use reader.take(section_length)
        let _section_length = VarUint32::deserialize(reader)?;
        let entries: Vec<ExportEntry> = CountedList::deserialize(reader)?.into_inner();
        Ok(ExportSection(entries))
    }   
}

pub struct CodeSection(Vec<FuncBody>);

impl CodeSection {
    pub fn bodies(&self) -> &[FuncBody] {
        &self.0
    }
}

impl Deserialize for CodeSection {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        // todo: maybe use reader.take(section_length)
        let _section_length = VarUint32::deserialize(reader)?;
        let entries: Vec<FuncBody> = CountedList::deserialize(reader)?.into_inner();
        Ok(CodeSection(entries))
    }   
}

impl Serialize for CodeSection {
    type Error = Error;
    
    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        let mut counted_writer = CountedWriter::new(writer);
        let data = self.0;
        let counted_list = CountedListWriter::<FuncBody, _>(
            data.len(),
            data.into_iter().map(Into::into),
        );
        counted_list.serialize(&mut counted_writer)?;
        counted_writer.done()?;
        Ok(())
    }
}

pub struct ElementSection(Vec<ElementSegment>);

impl ElementSection {
    pub fn new(entries: Vec<ElementSegment>) -> Self {
        ElementSection(entries)
    }

    pub fn entries(&self) -> &[ElementSegment] {
        &self.0
    }
}

impl Deserialize for ElementSection {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        // todo: maybe use reader.take(section_length)
        let _section_length = VarUint32::deserialize(reader)?;
        let entries: Vec<ElementSegment> = CountedList::deserialize(reader)?.into_inner();
        Ok(ElementSection(entries))
    }   
}

impl Serialize for ElementSection {
    type Error = Error;
    
    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        let mut counted_writer = CountedWriter::new(writer);
        let data = self.0;
        let counted_list = CountedListWriter::<ElementSegment, _>(
            data.len(),
            data.into_iter().map(Into::into),
        );
        counted_list.serialize(&mut counted_writer)?;
        counted_writer.done()?;
        Ok(())
    }
}

pub struct DataSection(Vec<DataSegment>);

impl DataSection {
    pub fn new(segments: Vec<DataSegment>) -> Self {
        DataSection(segments)
    }

    pub fn entries(&self) -> &[DataSegment] {
        &self.0
    }
}

impl Deserialize for DataSection {
    type Error = Error;

    fn deserialize<R: io::Read>(reader: &mut R) -> Result<Self, Self::Error> {
        // todo: maybe use reader.take(section_length)
        let _section_length = VarUint32::deserialize(reader)?;
        let entries: Vec<DataSegment> = CountedList::deserialize(reader)?.into_inner();
        Ok(DataSection(entries))
    }   
}

impl Serialize for DataSection {
    type Error = Error;
    
    fn serialize<W: io::Write>(self, writer: &mut W) -> Result<(), Self::Error> {
        let mut counted_writer = CountedWriter::new(writer);
        let data = self.0;
        let counted_list = CountedListWriter::<DataSegment, _>(
            data.len(),
            data.into_iter().map(Into::into),
        );
        counted_list.serialize(&mut counted_writer)?;
        counted_writer.done()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::super::{
        deserialize_buffer, deserialize_file, ValueType, InitExpr, DataSegment,
        serialize, ElementSegment,
    };
    use super::{Section, TypeSection, Type, DataSection, ElementSection};

    #[test]
    fn import_section() {
        let module = deserialize_file("./res/cases/v1/test5.wasm").expect("Should be deserialized");
        let mut found = false;
        for section in module.sections() {
            match section {
                &Section::Import(ref import_section) => { 
                    assert_eq!(25, import_section.entries().len());
                    found = true
                },
                _ => { }
            }
        }
        assert!(found, "There should be import section in test5.wasm");
    }

    fn functions_test_payload() -> Vec<u8> {
        vec![
            // functions section id
            0x03u8, 
            // functions section length
            0x87, 0x80, 0x80, 0x80, 0x0,
            // number of functions
            0x04, 
            // type reference 1
            0x01,
            // type reference 2
            0x86, 0x80, 0x00,
            // type reference 3
            0x09,            
            // type reference 4
            0x33
        ]
    }

    #[test]
    fn fn_section_detect() {
        let section: Section = 
            deserialize_buffer(functions_test_payload()).expect("section to be deserialized");

        match section {
            Section::Function(_) => {},
            _ => {
                panic!("Payload should be recognized as functions section")
            }
        }
    }

    #[test]
    fn fn_section_number() {
        let section: Section = 
            deserialize_buffer(functions_test_payload()).expect("section to be deserialized");

        match section {
            Section::Function(fn_section) => {
                assert_eq!(4, fn_section.entries().len(), "There should be 4 functions total");
            },
            _ => {
                // will be catched by dedicated test
            }
        }        
    }

    #[test]
    fn fn_section_ref() {
        let section: Section = 
            deserialize_buffer(functions_test_payload()).expect("section to be deserialized");

        match section {
            Section::Function(fn_section) => {
                assert_eq!(6, fn_section.entries()[1].type_ref());
            },
            _ => {
                // will be catched by dedicated test
            }
        }        
    }

    fn types_test_payload() -> Vec<u8> {
        vec![
            // section length
            148u8, 0x80, 0x80, 0x80, 0x0,
            
            // 2 functions
            130u8, 0x80, 0x80, 0x80, 0x0,
            // func 1, form =1
            0x01, 
            // param_count=1
            129u8, 0x80, 0x80, 0x80, 0x0,
                // first param
                0x7e, // i64
            // no return params
            0x00,

            // func 2, form=1
            0x01, 
            // param_count=1
            130u8, 0x80, 0x80, 0x80, 0x0,
                // first param
                0x7e, 
                // second param
                0x7d, 
            // return param (is_present, param_type)
            0x01, 0x7e
        ]
    }    

    #[test]
    fn type_section_len() {
        let type_section: TypeSection = 
            deserialize_buffer(types_test_payload()).expect("type_section be deserialized");

        assert_eq!(type_section.types().len(), 2);
    }

    #[test]
    fn type_section_infer() {
        let type_section: TypeSection = 
            deserialize_buffer(types_test_payload()).expect("type_section be deserialized");

        let t1 = match &type_section.types()[1] {
            &Type::Function(ref func_type) => func_type
        };

        assert_eq!(Some(ValueType::I64), t1.return_type());
        assert_eq!(2, t1.params().len());
    }

    fn export_payload() -> Vec<u8> {
        vec![
            // section id
            0x07,
            // section length
            148u8, 0x80, 0x80, 0x80, 0x0,
            // 6 entries
            134u8, 0x80, 0x80, 0x80, 0x0,
            // func "A", index 6 
            // [name_len(1-5 bytes), name_bytes(name_len, internal_kind(1byte), internal_index(1-5 bytes)])
            0x01, 0x41,  0x01, 0x86, 0x80, 0x00,
            // func "B", index 8
            0x01, 0x42,  0x01, 0x86, 0x00,
            // func "C", index 7
            0x01, 0x43,  0x01, 0x07,
            // memory "D", index 0
            0x01, 0x44,  0x02, 0x00,
            // func "E", index 1
            0x01, 0x45,  0x01, 0x01,
            // func "F", index 2
            0x01, 0x46,  0x01, 0x02
        ]
    }

 
    #[test]
    fn export_detect() {
        let section: Section = 
            deserialize_buffer(export_payload()).expect("section to be deserialized");

        match section {
            Section::Export(_) => {},
            _ => {
                panic!("Payload should be recognized as export section")
            }
        }
    }

    fn code_payload() -> Vec<u8> {
        vec![
            // sectionid
            0x0Au8,
            // section length, 32
            0x20,
            // body count
            0x01, 
            // body 1, length 30
            0x1E, 
            0x01, 0x01, 0x7F, // local i32 (one collection of length one of type i32)
            0x02, 0x7F, // block i32
                0x23, 0x00, // get_global 0
                0x21, 0x01, // set_local 1
                0x23, 0x00, // get_global 0
                0x20, 0x00, // get_local 0
                0x6A,       // i32.add
                0x24, 0x00, // set_global 0
                0x23, 0x00, // get_global 0
                0x41, 0x0F, // i32.const 15
                0x6A,       // i32.add
                0x41, 0x70, // i32.const -16
                0x71,       // i32.and
                0x24, 0x00, // set_global 0
                0x20, 0x01, // get_local 1
            0x0B,
            0x0B,
        ]
    }

    #[test]
    fn code_detect() {

        let section: Section = 
            deserialize_buffer(code_payload()).expect("section to be deserialized");

        match section {
            Section::Code(_) => {},
            _ => {
                panic!("Payload should be recognized as a code section")
            }
        }
    }

    fn data_payload() -> Vec<u8> {
        vec![
            0x0bu8,  // section id           
            19,      // 19 bytes overall
            0x01,    // number of segments
            0x00,    // index
            0x0b,    // just `end` op
            // 16x 0x00
            0x00, 0x00, 0x00, 0x00, 
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00
        ]
    }

    #[test]
    fn data_section_ser() {
        let data_section = DataSection::new(
            vec![DataSegment::new(0u32, InitExpr::empty(), vec![0u8; 16])]
        );

        let buf = serialize(data_section).expect("Data section to be serialized");

        assert_eq!(buf, vec![
            20u8, // 19 bytes overall
            0x01, // number of segments
            0x00, // index
            0x0b, // just `end` op
            16,   // value of length 16
            0x00, 0x00, 0x00, 0x00, // 16x 0x00 as in initialization
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00
        ]);
    }

    #[test]
    fn data_section_detect() {
        let section: Section = 
            deserialize_buffer(data_payload()).expect("section to be deserialized");

        match section {
            Section::Data(_) => {},
            _ => {
                panic!("Payload should be recognized as a data section")
            }
        }
    }

    #[test]
    fn element_section_ser() {
        let element_section = ElementSection::new(
            vec![ElementSegment::new(0u32, InitExpr::empty(), vec![0u32; 4])]
        );

        let buf = serialize(element_section).expect("Data section to be serialized");

        assert_eq!(buf, vec![
            08u8, // 8 bytes overall
            0x01, // number of segments
            0x00, // index
            0x0b, // just `end` op
            0x04, // 4 elements
            0x00, 0x00, 0x00, 0x00 // 4x 0x00 as in initialization
        ]);
    }    
}
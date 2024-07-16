use bitflags::bitflags;
use nom::{
    bytes::complete::take,
    combinator::map,
    multi::{count, many1},
    number::complete::{be_u16, be_u32},
    IResult,
};

use crate::instruction::Instruction;

#[derive(Debug)]
#[repr(u8)]
enum CpInfo {
    Class {
        name_index: u16,
    } = 7,
    FieldRef {
        class_index: u16,
        name_and_type_index: u16,
    } = 9,
    MethodRef {
        class_index: u16,
        name_and_type_index: u16,
    } = 10,
    InterfaceMethodRef {
        class_index: u16,
        name_and_type_index: u16,
    } = 11,
    String {
        string_index: u16,
    } = 8,
    Integer {
        bytes: u32,
    } = 3,
    Float {
        bytes: u32,
    } = 4,
    Long {
        high_bytes: u32,
        low_bytes: u32,
    } = 5,
    Double {
        high_bytes: u32,
        low_bytes: u32,
    } = 6,
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    } = 12,
    Utf8 {
        value: Box<str>,
    } = 1,
    MethodHandle {
        reference_kind: u8, // 1..9
        reference_index: u16,
    } = 15,
    MethodType {
        descriptor_index: u16,
    } = 16,
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    } = 18,
}

impl CpInfo {
    fn read(input: &[u8]) -> IResult<&[u8], CpInfo> {
        let (input, tag) = nom::number::complete::u8(input)?;
        match tag {
            7 => {
                let (input, name_index) = be_u16(input)?;
                Ok((input, CpInfo::Class { name_index }))
            }
            9 => {
                let (input, class_index) = be_u16(input)?;
                let (input, name_and_type_index) = be_u16(input)?;
                Ok((
                    input,
                    CpInfo::FieldRef {
                        class_index,
                        name_and_type_index,
                    },
                ))
            }
            10 => {
                let (input, class_index) = be_u16(input)?;
                let (input, name_and_type_index) = be_u16(input)?;
                Ok((
                    input,
                    CpInfo::MethodRef {
                        class_index,
                        name_and_type_index,
                    },
                ))
            }
            11 => {
                let (input, class_index) = be_u16(input)?;
                let (input, name_and_type_index) = be_u16(input)?;
                Ok((
                    input,
                    CpInfo::InterfaceMethodRef {
                        class_index,
                        name_and_type_index,
                    },
                ))
            }
            8 => {
                let (input, string_index) = be_u16(input)?;
                Ok((input, CpInfo::String { string_index }))
            }
            3 => {
                let (input, bytes) = be_u32(input)?;
                Ok((input, CpInfo::Integer { bytes }))
            }
            4 => {
                let (input, bytes) = be_u32(input)?;
                Ok((input, CpInfo::Float { bytes }))
            }
            5 => {
                let (input, high_bytes) = be_u32(input)?;
                let (input, low_bytes) = be_u32(input)?;
                Ok((
                    input,
                    CpInfo::Long {
                        high_bytes,
                        low_bytes,
                    },
                ))
            }
            6 => {
                let (input, high_bytes) = be_u32(input)?;
                let (input, low_bytes) = be_u32(input)?;
                Ok((
                    input,
                    CpInfo::Double {
                        high_bytes,
                        low_bytes,
                    },
                ))
            }
            12 => {
                let (input, name_index) = be_u16(input)?;
                let (input, descriptor_index) = be_u16(input)?;
                Ok((
                    input,
                    CpInfo::NameAndType {
                        name_index,
                        descriptor_index,
                    },
                ))
            }
            1 => {
                let (input, length) = be_u16(input)?;
                let (input, bytes) = take(length as usize)(input)?;

                let str = cesu8::from_java_cesu8(bytes).map_err(|err| {
                    nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify))
                })?;

                Ok((
                    input,
                    CpInfo::Utf8 {
                        value: str.to_string().into_boxed_str(),
                    },
                ))
            }
            15 => {
                let (input, reference_kind) = nom::number::complete::u8(input)?;
                let (input, reference_index) = be_u16(input)?;
                Ok((
                    input,
                    CpInfo::MethodHandle {
                        reference_kind,
                        reference_index,
                    },
                ))
            }
            16 => {
                let (input, descriptor_index) = be_u16(input)?;
                Ok((input, CpInfo::MethodType { descriptor_index }))
            }
            18 => {
                let (input, bootstrap_method_attr_index) = be_u16(input)?;
                let (input, name_and_type_index) = be_u16(input)?;
                Ok((
                    input,
                    CpInfo::InvokeDynamic {
                        bootstrap_method_attr_index,
                        name_and_type_index,
                    },
                ))
            }
            _ => unreachable!(),
        }
    }
}

bitflags! {
    #[derive(Debug)]
    pub struct FieldAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const VOLATILE = 0x0040;
        const TRANSIENT = 0x0080;
        const SYNTHETIC = 0x1000;
        const ENUM = 0x4000;
    }
}

impl FieldAccessFlags {
    fn read(input: &[u8]) -> IResult<&[u8], Self> {
        map(be_u16, Self::from_bits_retain)(input)
    }
}

#[derive(Debug)]
struct ExceptionTableEntry {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
}

impl ExceptionTableEntry {
    fn read(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, start_pc) = be_u16(input)?;
        let (input, end_pc) = be_u16(input)?;
        let (input, handler_pc) = be_u16(input)?;
        let (input, catch_type) = be_u16(input)?;
        Ok((
            input,
            Self {
                start_pc,
                end_pc,
                handler_pc,
                catch_type,
            },
        ))
    }
}

#[derive(Debug)]
struct LineNumberTableEntry {
    start_pc: u16,
    line_number: u16,
}

impl LineNumberTableEntry {
    fn read(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, start_pc) = be_u16(input)?;
        let (input, line_number) = be_u16(input)?;
        Ok((
            input,
            Self {
                start_pc,
                line_number,
            },
        ))
    }
}

#[derive(Debug)]
enum Attribute {
    Code {
        max_stack: u16,
        max_locals: u16,
        code_length: u32, // TODO: byte length. required?
        code: Box<[Instruction]>,
        exception_table: Box<[ExceptionTableEntry]>,
        attributes: Box<[AttributeInfo]>,
    },
    LineNumberTable {
        line_number_table: Box<[LineNumberTableEntry]>,
    },
    SourceFile {
        source_file_index: u16,
    },
    Unknown {
        bytes: Vec<u8>,
    },
}

#[derive(Debug)]
struct AttributeInfo {
    attribute_name_index: u16,
    attribute: Attribute,
}

impl AttributeInfo {
    fn read<'a>(input: &'a [u8], constant_pool: &[CpInfo]) -> IResult<&'a [u8], Self> {
        let (input, attribute_name_index) = be_u16(input)?;
        let (input, attribute_length) = be_u32(input)?;

        let attribute_name = match &constant_pool[attribute_name_index as usize] {
            CpInfo::Utf8 { value, .. } => value,
            _ => {
                return Err(nom::Err::Error(nom::error::Error {
                    input,
                    code: nom::error::ErrorKind::Fail,
                }))
            }
        };

        let (input, attribute) = match attribute_name.as_ref() {
            "Code" => {
                let (input, max_stack) = be_u16(input)?;
                let (input, max_locals) = be_u16(input)?;
                let (input, code_length) = be_u32(input)?;
                let (input, code) = take(code_length as usize)(input)?;

                let (_, instructions) = many1(Instruction::read)(code)?;

                let (input, exception_table_length) = be_u16(input)?;
                let (input, exception_table) =
                    count(ExceptionTableEntry::read, exception_table_length as usize)(input)?;
                let (input, attributes_count) = be_u16(input)?;
                let (input, attributes) = count(
                    |input| AttributeInfo::read(input, constant_pool),
                    attributes_count as usize,
                )(input)?;

                (
                    input,
                    Attribute::Code {
                        max_stack,
                        max_locals,
                        code_length,
                        code: instructions.into_boxed_slice(),
                        exception_table: exception_table.into_boxed_slice(),
                        attributes: attributes.into_boxed_slice(),
                    },
                )
            }
            "LineNumberTable" => {
                let (input, line_number_table_length) = be_u16(input)?;
                let (input, line_number_table) = count(
                    LineNumberTableEntry::read,
                    line_number_table_length as usize,
                )(input)?;
                (
                    input,
                    Attribute::LineNumberTable {
                        line_number_table: line_number_table.into_boxed_slice(),
                    },
                )
            }
            "SourceFile" => {
                let (input, source_file_index) = be_u16(input)?;
                (input, Attribute::SourceFile { source_file_index })
            }
            _ => {
                let (input, bytes) = take(attribute_length as usize)(input)?;
                (
                    input,
                    Attribute::Unknown {
                        bytes: bytes.to_vec(),
                    },
                )
            }
        };

        Ok((
            input,
            Self {
                attribute_name_index,
                attribute,
            },
        ))
    }
}

#[derive(Debug)]
struct FieldInfo {
    access_flags: FieldAccessFlags,
    name_index: u16,
    descriptor_index: u16,
    attributes: Box<[AttributeInfo]>,
}

impl FieldInfo {
    fn read<'a>(input: &'a [u8], constant_pool: &[CpInfo]) -> IResult<&'a [u8], Self> {
        let (input, access_flags) = FieldAccessFlags::read(input)?;
        let (input, name_index) = be_u16(input)?;
        let (input, descriptor_index) = be_u16(input)?;
        let (input, attributes_count) = be_u16(input)?;
        let (input, attributes) = count(
            |input| AttributeInfo::read(input, constant_pool),
            attributes_count as usize,
        )(input)?;

        Ok((
            input,
            Self {
                access_flags,
                name_index,
                descriptor_index,
                attributes: attributes.into_boxed_slice(),
            },
        ))
    }
}

bitflags! {
    #[derive(Debug)]
    pub struct MethodAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const SYNCHRONIZED = 0x0020;
        const BRIDGE = 0x0040;
        const VARARGS = 0x0080;
        const NATIVE = 0x0100;
        const ABSTRACT = 0x0400;
        const STRICT = 0x0800;
        const SYNTHETIC = 0x1000;
    }
}

impl MethodAccessFlags {
    fn read(input: &[u8]) -> IResult<&[u8], Self> {
        map(be_u16, Self::from_bits_retain)(input)
    }
}

#[derive(Debug)]
struct MethodInfo {
    access_flags: MethodAccessFlags,
    name_index: u16,
    descriptor_index: u16,
    attributes: Box<[AttributeInfo]>,
}

impl MethodInfo {
    fn read<'a>(input: &'a [u8], constant_pool: &[CpInfo]) -> IResult<&'a [u8], Self> {
        let (input, access_flags) = MethodAccessFlags::read(input)?;
        let (input, name_index) = be_u16(input)?;
        let (input, descriptor_index) = be_u16(input)?;
        let (input, attributes_count) = be_u16(input)?;
        let (input, attributes) = count(
            |input| AttributeInfo::read(input, constant_pool),
            attributes_count as usize,
        )(input)?;
        Ok((
            input,
            Self {
                access_flags,
                name_index,
                descriptor_index,
                attributes: attributes.into_boxed_slice(),
            },
        ))
    }
}

bitflags! {
    #[derive(Debug)]
    pub struct ClassAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const FINAL = 0x0010;
        const SUPER = 0x0020;
        const INTERFACE = 0x0200;
        const ABSTRACT = 0x0400;
        const SYNTHETIC = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM = 0x4000;
    }
}

impl ClassAccessFlags {
    fn read(input: &[u8]) -> IResult<&[u8], Self> {
        map(be_u16, Self::from_bits_retain)(input)
    }
}

#[derive(Debug)]
pub struct ClassFile {
    magic: u32,
    minor_version: u16,
    major_version: u16,
    constant_pool_count: u16,     // TODO: required to store?
    constant_pool: Box<[CpInfo]>, // 1..(constant_pool_count - 1)
    access_flags: ClassAccessFlags,
    this_class: u16,
    super_class: u16,
    interfaces: Vec<u16>,
    fields: Box<[FieldInfo]>,
    methods: Box<[MethodInfo]>,
    attributes: Box<[AttributeInfo]>,
}

impl ClassFile {
    pub fn read(input: &[u8]) -> IResult<&[u8], ClassFile> {
        let (input, magic) = be_u32(input)?;

        let (input, minor_version) = be_u16(input)?;
        let (input, major_version) = be_u16(input)?;

        let (input, constant_pool_count) = be_u16(input)?;

        let (input, mut constant_pool) =
            count(CpInfo::read, (constant_pool_count - 1) as usize)(input)?;
        constant_pool.insert(0, CpInfo::Class { name_index: 0 });

        let (input, access_flags) = ClassAccessFlags::read(input)?;

        let (input, this_class) = be_u16(input)?;
        let (input, super_class) = be_u16(input)?;

        let (input, interfaces_count) = be_u16(input)?;
        let (input, interfaces) = count(be_u16, interfaces_count as usize)(input)?;

        let (input, fields_count) = be_u16(input)?;
        let (input, fields) = count(
            |input| FieldInfo::read(input, &constant_pool),
            fields_count as usize,
        )(input)?;

        let (input, methods_count) = be_u16(input)?;
        let (input, methods) = count(
            |input| MethodInfo::read(input, &constant_pool),
            methods_count as usize,
        )(input)?;

        let (input, attributes_count) = be_u16(input)?;
        let (input, attributes) = count(
            |input| AttributeInfo::read(input, &constant_pool),
            attributes_count as usize,
        )(input)?;

        Ok((
            input,
            ClassFile {
                magic,
                minor_version,
                major_version,
                constant_pool_count,
                constant_pool: constant_pool.into_boxed_slice(),
                access_flags,
                this_class,
                super_class,
                interfaces,
                fields: fields.into_boxed_slice(),
                methods: methods.into_boxed_slice(),
                attributes: attributes.into_boxed_slice(),
            },
        ))
    }
}

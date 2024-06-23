use bitflags::bitflags;
use nom::{
    bytes::complete::take,
    combinator::map,
    multi::count,
    number::complete::{be_u16, be_u32},
    Err, IResult,
};

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
        length: u16,
        value: String,
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

                let mut str = String::new();

                let mut i = 0;
                while i < bytes.len() {
                    if (bytes[i] & 0b10000000) == 0b00000000 {
                        // Single byte codepoint
                        let b = bytes[i];

                        if b == 0 || (0xf0..=0xff).contains(&b) {
                            panic!("Invalid byte in UTF8 constant")
                        }

                        let c = char::from(b);
                        str.push(c);

                        i += 1;
                    } else if (bytes[i] & 0b11100000) == 0b11000000
                        && (bytes[i + 1] & 0b11000000) == 0b10000000
                    {
                        let x = bytes[i] as u32;
                        let y = bytes[i + 1] as u32;

                        let c = ((x & 0x1f) << 6) + (y & 0x3f);
                        let c = char::from_u32(c).expect("Java compiler HALP");
                        str.push(c);

                        i += 2;
                    } else if (bytes[i] & 0b11110000) == 0b11100000
                        && (bytes[i + 1] & 0b11000000) == 0b10000000
                        && (bytes[i + 2] & 0b11000000) == 0b10000000
                    {
                        let x = bytes[i] as u32;
                        let y = bytes[i + 1] as u32;
                        let z = bytes[i + 2] as u32;

                        let c = ((x & 0xf) << 12) + ((y & 0x3f) << 6) + (z & 0x3f);
                        let c = char::from_u32(c).expect("Java compiler HALP");
                        str.push(c);

                        i += 3;
                    } else if (bytes[i] & 0b11111111) == 0b11101101
                        && (bytes[i + 1] & 0b11110000) == 0b10100000
                        && (bytes[i + 2] & 0b11000000) == 0b10000000
                        && (bytes[i + 3] & 0b11111111) == 0b11101101
                        && (bytes[i + 4] & 0b11110000) == 0b10110000
                        && (bytes[i + 5] & 0b11000000) == 0b10000000
                    {
                        let _ = bytes[i] as u32;
                        let v = bytes[i + 1] as u32;
                        let w = bytes[i + 2] as u32;
                        let _ = bytes[i + 3] as u32;
                        let y = bytes[i + 4] as u32;
                        let z = bytes[i + 5] as u32;

                        let c = 0x10000
                            + ((v & 0x0f) << 16)
                            + ((w & 0x3f) << 10)
                            + ((y & 0x0f) << 6)
                            + (z & 0x3f);
                        let c = char::from_u32(c).expect("Java compiler HALP");
                        str.push(c);

                        i += 6
                    }
                }

                Ok((input, CpInfo::Utf8 { length, value: str }))
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
        map(be_u16, Self::from_bits_truncate)(input)
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
        code_length: u32,
        code: Vec<u8>,
        exception_table_length: u16,
        exception_table: Vec<ExceptionTableEntry>,
        attributes_count: u16,
        attributes: Vec<AttributeInfo>,
    },
    LineNumberTable {
        line_number_table_length: u16,
        line_number_table: Vec<LineNumberTableEntry>,
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
    attribute_length: u32,
    attribute: Attribute,
}

impl AttributeInfo {
    fn read<'a>(input: &'a [u8], constant_pool: &[CpInfo]) -> IResult<&'a [u8], Self> {
        let (input, attribute_name_index) = be_u16(input)?;
        let (input, attribute_length) = be_u32(input)?;

        let attribute_name = match &constant_pool[attribute_name_index as usize] {
            CpInfo::Utf8 { value, .. } => value,
            _ => {
                return Err(Err::Error(nom::error::Error {
                    input,
                    code: nom::error::ErrorKind::Fail,
                }))
            }
        };

        let (input, attribute) = match attribute_name.as_str() {
            "Code" => {
                let (input, max_stack) = be_u16(input)?;
                let (input, max_locals) = be_u16(input)?;
                let (input, code_length) = be_u32(input)?;
                let (input, code) = take(code_length as usize)(input)?;
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
                        code: code.to_vec(),
                        exception_table_length,
                        exception_table,
                        attributes_count,
                        attributes,
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
                        line_number_table_length,
                        line_number_table,
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
                attribute_length,
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
    attributes_count: u16,
    attributes: Vec<AttributeInfo>,
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
                attributes_count,
                attributes,
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
        map(be_u16, Self::from_bits_truncate)(input)
    }
}

#[derive(Debug)]
struct MethodInfo {
    access_flags: MethodAccessFlags,
    name_index: u16,
    descriptor_index: u16,
    attributes_count: u16,
    attributes: Vec<AttributeInfo>,
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
                attributes_count,
                attributes,
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
        map(be_u16, Self::from_bits_truncate)(input)
    }
}

#[derive(Debug)]
pub struct ClassFile {
    magic: u32,
    minor_version: u16,
    major_version: u16,
    constant_pool_count: u16,
    constant_pool: Vec<CpInfo>, // 1..(constant_pool_count - 1)
    access_flags: ClassAccessFlags,
    this_class: u16,
    super_class: u16,
    interfaces_count: u16,
    interfaces: Vec<u16>, // 0..interfaces_count
    fields_count: u16,
    fields: Vec<FieldInfo>, // 0..fields_count
    methods_count: u16,
    methods: Vec<MethodInfo>, // 0..methods_count
    attributes_count: u16,
    attributes: Vec<AttributeInfo>, // 0..attributes_count
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
                constant_pool,
                access_flags,
                this_class,
                super_class,
                interfaces_count,
                interfaces,
                fields_count,
                fields,
                methods_count,
                methods,
                attributes_count,
                attributes,
            },
        ))
    }
}

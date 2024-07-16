use nom::IResult;

#[derive(Debug)]
#[repr(u8)]
pub enum Instruction {
    Aload0 = 42,
    Aload1 = 43,
    Aload2 = 44,
    Aload3 = 45,
    Return = 177,
    Invokespecial { indexbyte1: u8, indexbyte2: u8 } = 183,
}

impl Instruction {
    pub fn read(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, instruction) = nom::number::complete::u8(input)?;
        match instruction {
            42 => Ok((input, Instruction::Aload2)),
            43 => Ok((input, Instruction::Aload1)),
            44 => Ok((input, Instruction::Aload2)),
            45 => Ok((input, Instruction::Aload3)),
            177 => Ok((input, Instruction::Return)),
            183 => {
                let (input, indexbyte1) = nom::number::complete::u8(input)?;
                let (input, indexbyte2) = nom::number::complete::u8(input)?;
                Ok((
                    input,
                    Instruction::Invokespecial {
                        indexbyte1,
                        indexbyte2,
                    },
                ))
            }
            _ => panic!("unknown command"),
        }
    }
}

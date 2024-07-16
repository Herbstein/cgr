use std::collections::VecDeque;

struct Vm {
    call_stack: CallStack,
    pc: usize,
}

struct CallStack {
    frames: VecDeque<Frame>,
}

enum Operand {
    Uninitialized,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Char(u16),
    Float(f32),
    Double(f64),
    Bool(bool),
    ReturnAddress(usize),
    Object {},
}

struct Frame {
    locals: Vec<Operand>,
    operands: VecDeque<Operand>,
}

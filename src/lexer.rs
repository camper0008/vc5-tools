use std::num::ParseIntError;

#[derive(Debug)]
pub enum Macro {
    Define { label: String, value: u16 },
    Label(String),
    SubLabel(String),
}

#[derive(Debug)]
pub enum Register {
    R0,
    R1,
}

#[derive(Debug)]
pub enum ValueVariant {
    Reference { label: String },
    Immediate(u16),
    Register(Register),
}

#[derive(Debug)]
pub struct Value {
    variant: ValueVariant,
    addressed: bool,
}

fn parse_digit(text: &str) -> Result<u16, Error> {
    if let Some(text) = text.strip_prefix("0x") {
        return u16::from_str_radix(text, 16)
            .map_err(|err| Error::BadNumber(text.to_string(), err));
    }
    if let Some(text) = text.strip_prefix("0b") {
        return u16::from_str_radix(text, 2).map_err(|err| Error::BadNumber(text.to_string(), err));
    }
    if let Some(text) = text.strip_prefix("-") {
        return i16::from_str_radix(text, 10)
            .map(|x| u16::from_ne_bytes(x.to_ne_bytes()))
            .map_err(|err| Error::BadNumber(text.to_string(), err));
    }
    u16::from_str_radix(text, 10)
        .map(|x| u16::from_ne_bytes(x.to_ne_bytes()))
        .map_err(|err| Error::BadNumber(text.to_string(), err))
}

fn parse_value(component: &str) -> Result<Value, Error> {
    let addressed = component.starts_with("[") && component.ends_with("]");
    let component = if addressed {
        &component[1..component.len() - 1]
    } else {
        component
    };
    let variant = match component {
        "r0" => ValueVariant::Register(Register::R0),
        component => parse_digit(component)
            .ok()
            .map(|x| ValueVariant::Immediate(x))
            .unwrap_or(ValueVariant::Reference {
                label: component.to_string(),
            }),
    };
    Ok(Value { addressed, variant })
}

#[derive(Debug)]
pub enum Instruction {
    Mov(Value, Value),
    MovByte(Value, Value),
    Lvcd(Value),
    Lkbd(Value),
    Jmp(Value),
    Hlt,
    Jnz(Value, Value),
    Reti,
    Add(Value, Value, Value),
    And(Value, Value, Value),
}

impl Instruction {
    pub fn width(&self) -> usize {
        match self {
            _ => todo!(),
        }
    }
}

#[derive(Debug)]
pub enum Line {
    Macro(Macro),
    Instruction(Instruction),
}

#[derive(Debug)]
pub enum Error {
    TooFewValues { expected: usize, received: usize },
    TooManyValues { expected: usize, received: usize },
    UnknownStatement(String, String),
    BadNumber(String, ParseIntError),
    UnknownMovModifier(String),
}

fn assert_components_len(expected: usize, received: usize) -> Result<(), Error> {
    if received > expected {
        Err(Error::TooManyValues { expected, received })
    } else if received < expected {
        Err(Error::TooFewValues { expected, received })
    } else {
        Ok(())
    }
}

fn parse_label(components: Vec<&str>) -> Result<Line, Error> {
    assert_components_len(1, components.len())?;
    let label = components[0];
    let without_cruft = label.trim_start_matches(".").trim_end_matches(":");
    if label.starts_with(".") {
        Ok(Line::Macro(Macro::SubLabel(without_cruft.to_string())))
    } else {
        Ok(Line::Macro(Macro::Label(without_cruft.to_string())))
    }
}

fn parse_define(components: Vec<&str>) -> Result<Line, Error> {
    assert_components_len(3, components.len())?;
    let label = components[1].to_string();
    let value = parse_digit(components[2])?;
    Ok(Line::Macro(Macro::Define { label, value }))
}

fn parse_0(components: Vec<&str>) -> Result<Line, Error> {
    let value = match components[0] {
        "hlt" => Instruction::Hlt,
        "reti" => Instruction::Reti,
        _ => reached_unknown(components)?,
    };
    Ok(Line::Instruction(value))
}

fn parse_1(components: Vec<&str>) -> Result<Line, Error> {
    let _0 = parse_value(components[1])?;
    let con = match components[0] {
        "jmp" => Instruction::Jmp,
        "lvcd" => Instruction::Lvcd,
        "lkbd" => Instruction::Lkbd,
        _ => reached_unknown(components)?,
    };
    Ok(Line::Instruction(con(_0)))
}

fn parse_2(components: Vec<&str>) -> Result<Line, Error> {
    let _0 = parse_value(components[1])?;
    let _1 = parse_value(components[2])?;
    let con = match components[0] {
        "jnz" => Instruction::Jnz,
        "mov" => Instruction::Mov,
        _ => reached_unknown(components)?,
    };
    Ok(Line::Instruction(con(_0, _1)))
}

fn parse_3(components: Vec<&str>) -> Result<Line, Error> {
    if components[0] == "mov" {
        let mut components = components;
        let modifier = components.remove(1);
        if modifier != "byte" {
            return Err(Error::UnknownMovModifier(modifier.to_string()));
        }
        let _0 = parse_value(components[1])?;
        let _1 = parse_value(components[2])?;

        return Ok(Line::Instruction(Instruction::MovByte(_0, _1)));
    }
    let _0 = parse_value(components[1])?;
    let _1 = parse_value(components[2])?;
    let _2 = parse_value(components[3])?;
    let con = match components[0] {
        "add" => Instruction::Add,
        "and" => Instruction::And,
        _ => reached_unknown(components)?,
    };
    Ok(Line::Instruction(con(_0, _1, _2)))
}

fn reached_unknown<T>(components: Vec<&str>) -> Result<T, Error> {
    Err(Error::UnknownStatement(
        components[0].to_string(),
        components.join(" "),
    ))
}

pub fn parse_line(components: Vec<&str>) -> Result<Line, Error> {
    if components[0].ends_with(":") {
        return parse_label(components);
    }
    if components[0].starts_with("%define") {
        return parse_define(components);
    }
    let parsers = [parse_0, parse_1, parse_2, parse_3];
    let picked = parsers[components.len() - 1];

    picked(components)
}

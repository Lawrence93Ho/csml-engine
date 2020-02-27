use crate::data::primitive::float::PrimitiveFloat;
use crate::data::primitive::object::PrimitiveObject;
use crate::data::primitive::string::PrimitiveString;
use crate::data::primitive::boolean::PrimitiveBoolean;
use crate::data::primitive::tools::check_division_by_zero_i64;
use crate::data::primitive::tools::check_usage;
use crate::data::primitive::Right;
use crate::data::primitive::{Primitive, PrimitiveType};
use crate::data::{ast::Interval, memories::MemoryType, message::Message, Literal};
use crate::error_format::ErrorInfo;
use lazy_static::*;
use std::cmp::Ordering;
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// DATA STRUCTURES
////////////////////////////////////////////////////////////////////////////////

type PrimitiveMethod =
    fn(int: &mut PrimitiveInt, args: &[Literal], interval: Interval) -> Result<Literal, ErrorInfo>;

lazy_static! {
    static ref FUNCTIONS: HashMap<&'static str, (PrimitiveMethod, Right)> = {
        let mut map = HashMap::new();

        map.insert("type_of", (type_of as PrimitiveMethod, Right::Read));
        map.insert("to_string", (to_string as PrimitiveMethod, Right::Read));
        map.insert("abs", (abs as PrimitiveMethod, Right::Read));
        map.insert("cos", (cos as PrimitiveMethod, Right::Read));
        map.insert("pow", (pow as PrimitiveMethod, Right::Read));
        map.insert("floor", (floor as PrimitiveMethod, Right::Read));
        map.insert("ceil", (ceil as PrimitiveMethod, Right::Read));
        map.insert("round", (round as PrimitiveMethod, Right::Read));
        map.insert("sin", (sin as PrimitiveMethod, Right::Read));
        map.insert("sqrt", (sqrt as PrimitiveMethod, Right::Read));
        map.insert("tan", (tan as PrimitiveMethod, Right::Read));
        map.insert("is_number", (is_number as PrimitiveMethod, Right::Read));
        map.insert("to_int", (to_int as PrimitiveMethod, Right::Read));
        map.insert("to_float", (to_float as PrimitiveMethod, Right::Read));

        map
    };
}

#[derive(PartialEq, Debug, Clone)]
pub struct PrimitiveInt {
    pub value: i64,
}

////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

/// type_of() -> Primitive<String>
fn type_of(
    _int: &mut PrimitiveInt,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "type_of()", interval)?;

    Ok(PrimitiveString::get_literal("string", "int", interval))
}

/// to_string() -> Primitive<String>
fn to_string(
    int: &mut PrimitiveInt,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "to_string()", interval)?;

    Ok(PrimitiveString::get_literal(
        "string",
        &int.to_string(),
        interval,
    ))
}

/// abs() -> Primitive<Int>
fn abs(
    int: &mut PrimitiveInt,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "abs()", interval)?;

    let result = int.value as f64;
    let result = result.abs();
    let result = result as i64;

    Ok(PrimitiveInt::get_literal("int", result, interval))
}

/// cos() -> Primitive<Int>
fn cos(
    int: &mut PrimitiveInt,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "cos()", interval)?;

    let result = int.value as f64;
    let result = result.cos();
    let result = result as i64;

    Ok(PrimitiveInt::get_literal("int", result, interval))
}

/// pow(Primitive<String> || Primitive<Float>) -> Primitive<Int>
fn pow(int: &mut PrimitiveInt, args: &[Literal], interval: Interval) -> Result<Literal, ErrorInfo> {
    check_usage(args, 1, "pow(Primitive<Int || Float>)", interval)?;

    let literal = match args.get(0) {
        Some(res) => res,
        None => {
            return Err(ErrorInfo {
                message: "usage: need to have one parameter".to_owned(),
                interval,
            });
        }
    };

    if let Ok(res) = Literal::get_value::<f64>(&literal.primitive) {
        let res = *res as u32;
        let result = int.value.pow(res);

        return Ok(PrimitiveInt::get_literal("int", result, interval));
    }
    if let Ok(res) = Literal::get_value::<i64>(&literal.primitive) {
        let result = int.value.pow(*res as u32);

        return Ok(PrimitiveInt::get_literal("int", result, interval));
    }

    Err(ErrorInfo {
        message: "usage: parameter must be of type float or int".to_owned(),
        interval,
    })
}

/// floor() -> Primitive<Int>
fn floor(
    int: &mut PrimitiveInt,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "floor()", interval)?;

    let result = int.value as f64;
    let result = result.floor();
    let result = result as i64;

    Ok(PrimitiveInt::get_literal("int", result, interval))
}

/// ceil() -> Primitive<Int>
fn ceil(
    int: &mut PrimitiveInt,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "ceil()", interval)?;

    let result = int.value as f64;
    let result = result.ceil();
    let result = result as i64;

    Ok(PrimitiveInt::get_literal("int", result, interval))
}

/// round() -> Primitive<Int>
fn round(
    int: &mut PrimitiveInt,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "round()", interval)?;

    let result = int.value as f64;
    let result = result.round();
    let result = result as i64;

    Ok(PrimitiveInt::get_literal("int", result, interval))
}

/// sin() -> Primitive<Int>
fn sin(
    int: &mut PrimitiveInt,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "sin()", interval)?;

    let result = int.value as f64;
    let result = result.sin();
    let result = result as i64;

    Ok(PrimitiveInt::get_literal("int", result, interval))
}

/// sqrt() -> Primitive<Int>
fn sqrt(
    int: &mut PrimitiveInt,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "sqrt()", interval)?;

    let result = int.value as f64;
    let result = result.sqrt();
    let result = result as i64;

    Ok(PrimitiveInt::get_literal("int", result, interval))
}

/// tan() -> Primitive<Int>
fn tan(
    int: &mut PrimitiveInt,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "tan()", interval)?;

    let result = int.value as f64;
    let result = result.tan();
    let result = result as i64;

    Ok(PrimitiveInt::get_literal("int", result, interval))
}

/// is_number() -> Primitive<Boolean>
fn is_number(
    _int: &mut PrimitiveInt,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "is_number()", interval)?;

    Ok(PrimitiveBoolean::get_literal("boolean", true, interval))
}

/// to_int() -> Primitive<Int>
fn to_int(
    int: &mut PrimitiveInt,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "to_int()", interval)?;

    Ok(PrimitiveInt::get_literal("int", int.value, interval))
}

/// to_float() -> Primitive<Float>
fn to_float(
    int: &mut PrimitiveInt,
    args: &[Literal],
    interval: Interval,
) -> Result<Literal, ErrorInfo> {
    check_usage(args, 0, "to_float()", interval)?;

    Ok(PrimitiveFloat::get_literal("float", int.value as f64, interval))
}

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

impl PrimitiveInt {
    pub fn new(value: i64) -> Self {
        Self { value }
    }

    pub fn get_literal(content_type: &str, int: i64, interval: Interval) -> Literal {
        let primitive = Box::new(PrimitiveInt::new(int));

        Literal {
            content_type: content_type.to_owned(),
            primitive,
            interval,
        }
    }
}

impl Primitive for PrimitiveInt {
    fn do_exec(
        &mut self,
        name: &str,
        args: &[Literal],
        interval: Interval,
        _mem_type: &MemoryType,
    ) -> Result<(Literal, Right), ErrorInfo> {
        if let Some((f, right)) = FUNCTIONS.get(name) {
            let res = f(self, args, interval)?;

            return Ok((res, *right));
        }

        Err(ErrorInfo {
            message: format!("unknown method '{}' for type Int", name),
            interval,
        })
    }

    fn is_eq(&self, other: &dyn Primitive) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.value == other.value
        } else {
            false
        }
    }

    fn is_cmp(&self, other: &dyn Primitive) -> Option<Ordering> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            return self.value.partial_cmp(&other.value);
        }

        None
    }

    fn do_add(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let result = self.value + other.value;

            return Ok(Box::new(PrimitiveInt::new(result)));
        }

        Err(ErrorInfo {
            message: "[!] Add: Illegal operation".to_owned(),
            interval: Interval { column: 0, line: 0 },
        })
    }

    fn do_sub(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let result = self.value - other.value;

            return Ok(Box::new(PrimitiveInt::new(result)));
        }

        Err(ErrorInfo {
            message: "[!] Sub: Illegal operation".to_owned(),
            interval: Interval { column: 0, line: 0 },
        })
    }

    fn do_div(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            check_division_by_zero_i64(self.value, other.value)?;

            if self.value % other.value != 0 {
                let result = self.value as f64 / other.value as f64;

                return Ok(Box::new(PrimitiveFloat::new(result)));
            } else {
                let result = self.value / other.value;

                return Ok(Box::new(PrimitiveInt::new(result)));
            }
        }

        Err(ErrorInfo {
            message: "[!] Div: Illegal operation".to_owned(),
            interval: Interval { column: 0, line: 0 },
        })
    }

    fn do_mul(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let result = self.value * other.value;

            return Ok(Box::new(PrimitiveInt::new(result)));
        }

        Err(ErrorInfo {
            message: "[!] Mul: Illegal operation".to_owned(),
            interval: Interval { column: 0, line: 0 },
        })
    }

    fn do_rem(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let result = self.value % other.value;

            return Ok(Box::new(PrimitiveInt::new(result)));
        }

        Err(ErrorInfo {
            message: "[!] Rem: Illegal operation".to_owned(),
            interval: Interval { column: 0, line: 0 },
        })
    }

    fn do_bitand(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let result = self.value & other.value;

            return Ok(Box::new(PrimitiveInt::new(result)));
        }

        Err(ErrorInfo {
            message: "[!] BitAnd: Illegal operation".to_owned(),
            interval: Interval { column: 0, line: 0 },
        })
    }

    fn do_bitor(&self, other: &dyn Primitive) -> Result<Box<dyn Primitive>, ErrorInfo> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            let result = self.value | other.value;

            return Ok(Box::new(PrimitiveInt::new(result)));
        }

        Err(ErrorInfo {
            message: "[!] BitOr: Illegal operation".to_owned(),
            interval: Interval { column: 0, line: 0 },
        })
    }

    fn as_debug(&self) -> &dyn std::fmt::Debug {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_type(&self) -> PrimitiveType {
        PrimitiveType::PrimitiveInt
    }

    fn as_box_clone(&self) -> Box<dyn Primitive> {
        Box::new((*self).clone())
    }

    fn to_json(&self) -> serde_json::Value {
        serde_json::json!(self.value)
    }

    fn to_string(&self) -> String {
        self.value.to_string()
    }

    fn as_bool(&self) -> bool {
        self.value.is_positive()
    }

    fn get_value(&self) -> &dyn std::any::Any {
        &self.value
    }

    fn get_mut_value(&mut self) -> &mut dyn std::any::Any {
        &mut self.value
    }

    fn to_msg(&self, _content_type: String) -> Message {
        let mut hashmap: HashMap<String, Literal> = HashMap::new();

        hashmap.insert(
            "text".to_owned(),
            Literal {
                content_type: "int".to_owned(),
                primitive: Box::new(PrimitiveString::new(&self.to_string())),
                interval: Interval { column: 0, line: 0 },
            },
        );

        let result =
            PrimitiveObject::get_literal("text", &hashmap, Interval { column: 0, line: 0 });

        Message {
            content_type: result.content_type,
            content: result.primitive.to_json(),
        }
    }
}

use std::collections::HashMap;
use std::mem;
use std::fmt;
use std::str;

use serialize;
use {Value, Table, Array, String, Integer, Float, Boolean, Parser};

/// A structure to transform Rust values into TOML values.
///
/// This encoder implements the serialization `Encoder` interface, allowing
/// `Encodable` rust types to be fed into the encoder. The output of this
/// encoder is a TOML `Table` structure. The resulting TOML can be stringified
/// if necessary.
///
/// # Example
///
/// ```
/// extern crate serialize;
/// extern crate toml;
///
/// # fn main() {
/// use toml::{Encoder, Integer};
/// use serialize::Encodable;
///
/// #[deriving(Encodable)]
/// struct MyStruct { foo: int, bar: String }
/// let my_struct = MyStruct { foo: 4, bar: "hello!".to_string() };
///
/// let mut e = Encoder::new();
/// my_struct.encode(&mut e).unwrap();
///
/// assert_eq!(e.toml.find_equiv(&"foo"), Some(&Integer(4)))
/// # }
/// ```
pub struct Encoder {
    /// Output TOML that is emitted. The current version of this encoder forces
    /// the top-level representation of a structure to be a table.
    ///
    /// This field can be used to extract the return value after feeding a value
    /// into this `Encoder`.
    pub toml: Table,
    state: EncoderState,
}

/// A structure to transform TOML values into Rust values.
///
/// This decoder implements the serialization `Decoder` interface, allowing
/// `Decodable` types to be generated by this decoder. The input is any
/// arbitrary TOML value.
pub struct Decoder {
    /// The TOML value left over after decoding. This can be used to inspect
    /// whether fields were decoded or not.
    pub toml: Option<Value>,
    cur_field: Option<String>,
}

/// Enumeration of errors which can occur while encoding a rust value into a
/// TOML value.
#[deriving(Show)]
pub enum Error {
    /// Indication that a key was needed when a value was emitted, but no key
    /// was previously emitted.
    NeedsKey,
    /// Indication that a key was emitted, but not value was emitted.
    NoValue,
    /// Indicates that a map key was attempted to be emitted at an invalid
    /// location.
    InvalidMapKeyLocation,
    /// Indicates that a type other than a string was attempted to be used as a
    /// map key type.
    InvalidMapKeyType,
}

/// Description for errors which can occur while decoding a type.
pub struct DecodeError {
    /// Field that this error applies to.
    pub field: Option<String>,
    /// The type of error which occurred while decoding,
    pub kind: DecodeErrorKind,
}

/// Enumeration of possible errors which can occur while decoding a structure.
pub enum DecodeErrorKind {
    /// A field was expected, but none was found.
    ExpectedField(/* type */ &'static str),
    /// A field was found, but it had the wrong type.
    ExpectedType(/* expected */ &'static str, /* found */ &'static str),
    /// The nth map key was expected, but none was found.
    ExpectedMapKey(uint),
    /// The nth map element was expected, but none was found.
    ExpectedMapElement(uint),
    /// An enum decoding was requested, but no variants were supplied
    NoEnumVariants,
    /// The unit type was being decoded, but a non-zero length string was found
    NilTooLong
}

#[deriving(PartialEq, Show)]
enum EncoderState {
    Start,
    NextKey(String),
    NextArray(Vec<Value>),
    NextMapKey,
}

/// Encodes an encodable value into a TOML value.
///
/// This function expects the type given to represent a TOML table in some form.
/// If encoding encounters an error, then this function will fail the task.
pub fn encode<T: serialize::Encodable<Encoder, Error>>(t: &T) -> Value {
    let mut e = Encoder::new();
    t.encode(&mut e).unwrap();
    Table(e.toml)
}

/// Encodes an encodable value into a TOML string.
///
/// This function expects the type given to represent a TOML table in some form.
/// If encoding encounters an error, then this function will fail the task.
pub fn encode_str<T: serialize::Encodable<Encoder, Error>>(t: &T) -> String {
    format!("{}", encode(t))
}

impl Encoder {
    /// Constructs a new encoder which will emit to the given output stream.
    pub fn new() -> Encoder {
        Encoder { state: Start, toml: HashMap::new() }
    }

    fn emit_value(&mut self, v: Value) -> Result<(), Error> {
        match mem::replace(&mut self.state, Start) {
            NextKey(key) => { self.toml.insert(key, v); Ok(()) }
            NextArray(mut vec) => {
                // TODO: validate types
                vec.push(v);
                self.state = NextArray(vec);
                Ok(())
            }
            NextMapKey => {
                match v {
                    String(s) => { self.state = NextKey(s); Ok(()) }
                    _ => Err(InvalidMapKeyType)
                }
            }
            _ => Err(NeedsKey)
        }
    }
}

impl serialize::Encoder<Error> for Encoder {
    fn emit_nil(&mut self) -> Result<(), Error> { Ok(()) }
    fn emit_uint(&mut self, v: uint) -> Result<(), Error> {
        self.emit_i64(v as i64)
    }
    fn emit_u8(&mut self, v: u8) -> Result<(), Error> {
        self.emit_i64(v as i64)
    }
    fn emit_u16(&mut self, v: u16) -> Result<(), Error> {
        self.emit_i64(v as i64)
    }
    fn emit_u32(&mut self, v: u32) -> Result<(), Error> {
        self.emit_i64(v as i64)
    }
    fn emit_u64(&mut self, v: u64) -> Result<(), Error> {
        self.emit_i64(v as i64)
    }
    fn emit_int(&mut self, v: int) -> Result<(), Error> {
        self.emit_i64(v as i64)
    }
    fn emit_i8(&mut self, v: i8) -> Result<(), Error> {
        self.emit_i64(v as i64)
    }
    fn emit_i16(&mut self, v: i16) -> Result<(), Error> {
        self.emit_i64(v as i64)
    }
    fn emit_i32(&mut self, v: i32) -> Result<(), Error> {
        self.emit_i64(v as i64)
    }
    fn emit_i64(&mut self, v: i64) -> Result<(), Error> {
        self.emit_value(Integer(v))
    }
    fn emit_bool(&mut self, v: bool) -> Result<(), Error> {
        self.emit_value(Boolean(v))
    }
    fn emit_f32(&mut self, v: f32) -> Result<(), Error> { self.emit_f64(v as f64) }
    fn emit_f64(&mut self, v: f64) -> Result<(), Error> {
        self.emit_value(Float(v))
    }
    fn emit_char(&mut self, v: char) -> Result<(), Error> {
        self.emit_str(v.to_str().as_slice())
    }
    fn emit_str(&mut self, v: &str) -> Result<(), Error> {
        self.emit_value(String(v.to_str()))
    }
    fn emit_enum(&mut self, _name: &str,
                 f: |&mut Encoder| -> Result<(), Error>) -> Result<(), Error> {
        f(self)
    }
    fn emit_enum_variant(&mut self, _v_name: &str, _v_id: uint, _len: uint,
                         f: |&mut Encoder| -> Result<(), Error>)
        -> Result<(), Error>
    {
        f(self)
    }
    fn emit_enum_variant_arg(&mut self, _a_idx: uint,
                             f: |&mut Encoder| -> Result<(), Error>)
        -> Result<(), Error>
    {
        f(self)
    }
    fn emit_enum_struct_variant(&mut self, _v_name: &str, _v_id: uint,
                                _len: uint,
                                _f: |&mut Encoder| -> Result<(), Error>)
        -> Result<(), Error>
    {
        fail!()
    }
    fn emit_enum_struct_variant_field(&mut self, _f_name: &str, _f_idx: uint,
                                      _f: |&mut Encoder| -> Result<(), Error>)
        -> Result<(), Error>
    {
        fail!()
    }
    fn emit_struct(&mut self, _name: &str, _len: uint,
                   f: |&mut Encoder| -> Result<(), Error>) -> Result<(), Error> {
        match mem::replace(&mut self.state, Start) {
            NextKey(key) => {
                let mut nested = Encoder::new();
                try!(f(&mut nested));
                self.toml.insert(key, Table(nested.toml));
                Ok(())
            }
            NextArray(mut arr) => {
                let mut nested = Encoder::new();
                try!(f(&mut nested));
                arr.push(Table(nested.toml));
                self.state = NextArray(arr);
                Ok(())
            }
            Start => f(self),
            NextMapKey => Err(InvalidMapKeyLocation),
        }
    }
    fn emit_struct_field(&mut self, f_name: &str, _f_idx: uint,
                         f: |&mut Encoder| -> Result<(), Error>)
        -> Result<(), Error>
    {
        let old = mem::replace(&mut self.state, NextKey(f_name.to_str()));
        try!(f(self));
        if self.state != Start {
            println!("{}", self.state);
            return Err(NoValue)
        }
        self.state = old;
        Ok(())
    }
    fn emit_tuple(&mut self, len: uint,
                  f: |&mut Encoder| -> Result<(), Error>) -> Result<(), Error> {
        self.emit_seq(len, f)
    }
    fn emit_tuple_arg(&mut self, idx: uint,
                      f: |&mut Encoder| -> Result<(), Error>)
        -> Result<(), Error>
    {
        self.emit_seq_elt(idx, f)
    }
    fn emit_tuple_struct(&mut self, _name: &str, _len: uint,
                         _f: |&mut Encoder| -> Result<(), Error>)
        -> Result<(), Error>
    {
        unimplemented!()
    }
    fn emit_tuple_struct_arg(&mut self, _f_idx: uint,
                             _f: |&mut Encoder| -> Result<(), Error>)
        -> Result<(), Error>
    {
        unimplemented!()
    }
    fn emit_option(&mut self,
                   f: |&mut Encoder| -> Result<(), Error>)
        -> Result<(), Error>
    {
        f(self)
    }
    fn emit_option_none(&mut self) -> Result<(), Error> {
        match mem::replace(&mut self.state, Start) {
            Start => unreachable!(),
            NextKey(_) => Ok(()),
            NextArray(..) => fail!("how to encode None in an array?"),
            NextMapKey => Err(InvalidMapKeyLocation),
        }
    }
    fn emit_option_some(&mut self,
                        f: |&mut Encoder| -> Result<(), Error>)
        -> Result<(), Error>
    {
        f(self)
    }
    fn emit_seq(&mut self, _len: uint,
                f: |this: &mut Encoder| -> Result<(), Error>)
        -> Result<(), Error>
    {
        let old = mem::replace(&mut self.state, NextArray(Vec::new()));
        try!(f(self));
        match mem::replace(&mut self.state, old) {
            NextArray(v) => self.emit_value(Array(v)),
            _ => unreachable!(),
        }
    }
    fn emit_seq_elt(&mut self, _idx: uint,
                    f: |this: &mut Encoder| -> Result<(), Error>)
        -> Result<(), Error>
    {
        f(self)
    }
    fn emit_map(&mut self, len: uint,
                f: |&mut Encoder| -> Result<(), Error>) -> Result<(), Error> {
        self.emit_struct("foo", len, f)
    }
    fn emit_map_elt_key(&mut self, _idx: uint,
                        f: |&mut Encoder| -> Result<(), Error>)
        -> Result<(), Error>
    {
        match mem::replace(&mut self.state, NextMapKey) {
            Start => {}
            _ => return Err(InvalidMapKeyLocation),
        }
        try!(f(self));
        match self.state {
            NextKey(_) => Ok(()),
            _ => Err(InvalidMapKeyLocation),
        }
    }
    fn emit_map_elt_val(&mut self, _idx: uint,
                        f: |&mut Encoder| -> Result<(), Error>)
        -> Result<(), Error>
    {
        f(self)
    }
}

/// Decodes a TOML value into a decodable type.
///
/// This function will consume the given TOML value and attempt to decode it
/// into the type specified. If decoding fails, `None` will be returned. If a
/// finer-grained error is desired, then it is recommended to use `Decodable`
/// directly.
pub fn decode<T: serialize::Decodable<Decoder, DecodeError>>(toml: Value)
    -> Option<T>
{
    serialize::Decodable::decode(&mut Decoder::new(toml)).ok()
}

/// Decodes a string into a toml-encoded value.
///
/// This function will parse the given string into a TOML value, and then parse
/// the TOML value into the desired type. If any error occurs `None` is return.
/// If more fine-grained errors are desired, these steps should be driven
/// manually.
pub fn decode_str<T: serialize::Decodable<Decoder, DecodeError>>(s: &str)
    -> Option<T>
{
    Parser::new(s).parse().and_then(|t| decode(Table(t)))
}

impl Decoder {
    /// Creates a new decoder, consuming the TOML value to decode.
    ///
    /// This decoder can be passed to the `Decodable` methods or driven
    /// manually.
    pub fn new(toml: Value) -> Decoder {
        Decoder { toml: Some(toml), cur_field: None }
    }

    fn sub_decoder(&self, toml: Option<Value>, field: &str) -> Decoder {
        Decoder {
            toml: toml,
            cur_field: if field.len() == 0 {
                self.cur_field.clone()
            } else {
                match self.cur_field {
                    None => Some(field.to_string()),
                    Some(ref s) => Some(format!("{}.{}", s, field))
                }
            }
        }
    }

    fn err(&self, kind: DecodeErrorKind) -> DecodeError {
        DecodeError {
            field: self.cur_field.clone(),
            kind: kind,
        }
    }

    fn mismatch(&self, expected: &'static str,
                found: &Option<Value>) -> DecodeError{
        match *found {
            Some(ref val) => self.err(ExpectedType(expected, val.type_str())),
            None => self.err(ExpectedField(expected)),
        }
    }
}

impl serialize::Decoder<DecodeError> for Decoder {
    fn read_nil(&mut self) -> Result<(), DecodeError> {
        match self.toml {
            Some(String(ref s)) if s.len() == 0 => {}
            Some(String(..)) => return Err(self.err(NilTooLong)),
            ref found => return Err(self.mismatch("string", found)),
        }
        self.toml.take();
        Ok(())
    }
    fn read_uint(&mut self) -> Result<uint, DecodeError> {
        self.read_i64().map(|i| i as uint)
    }
    fn read_u64(&mut self) -> Result<u64, DecodeError> {
        self.read_i64().map(|i| i as u64)
    }
    fn read_u32(&mut self) -> Result<u32, DecodeError> {
        self.read_i64().map(|i| i as u32)
    }
    fn read_u16(&mut self) -> Result<u16, DecodeError> {
        self.read_i64().map(|i| i as u16)
    }
    fn read_u8(&mut self) -> Result<u8, DecodeError> {
        self.read_i64().map(|i| i as u8)
    }
    fn read_int(&mut self) -> Result<int, DecodeError> {
        self.read_i64().map(|i| i as int)
    }
    fn read_i64(&mut self) -> Result<i64, DecodeError> {
        match self.toml {
            Some(Integer(i)) => { self.toml.take(); Ok(i) }
            ref found => Err(self.mismatch("integer", found)),
        }
    }
    fn read_i32(&mut self) -> Result<i32, DecodeError> {
        self.read_i64().map(|i| i as i32)
    }
    fn read_i16(&mut self) -> Result<i16, DecodeError> {
        self.read_i64().map(|i| i as i16)
    }
    fn read_i8(&mut self) -> Result<i8, DecodeError> {
        self.read_i64().map(|i| i as i8)
    }
    fn read_bool(&mut self) -> Result<bool, DecodeError> {
        match self.toml {
            Some(Boolean(b)) => { self.toml.take(); Ok(b) }
            ref found => Err(self.mismatch("bool", found)),
        }
    }
    fn read_f64(&mut self) -> Result<f64, DecodeError> {
        match self.toml {
            Some(Float(f)) => Ok(f),
            ref found => Err(self.mismatch("float", found)),
        }
    }
    fn read_f32(&mut self) -> Result<f32, DecodeError> {
        self.read_f64().map(|f| f as f32)
    }
    fn read_char(&mut self) -> Result<char, DecodeError> {
        let ch = match self.toml {
            Some(String(ref s)) if s.as_slice().char_len() == 1 =>
                s.as_slice().char_at(0),
            ref found => return Err(self.mismatch("string", found)),
        };
        self.toml.take();
        Ok(ch)
    }
    fn read_str(&mut self) -> Result<String, DecodeError> {
        match self.toml.take() {
            Some(String(s)) => Ok(s),
            found => {
                let err = Err(self.mismatch("string", &found));
                self.toml = found;
                err
            }
        }
    }

    // Compound types:
    fn read_enum<T>(&mut self, _name: &str,
                    f: |&mut Decoder| -> Result<T, DecodeError>)
        -> Result<T, DecodeError>
    {
        f(self)
    }

    fn read_enum_variant<T>(&mut self,
                            names: &[&str],
                            f: |&mut Decoder, uint| -> Result<T, DecodeError>)
                            -> Result<T, DecodeError> {
        let mut first_error = None;
        for i in range(0, names.len()) {
            let mut d = self.sub_decoder(self.toml.clone(), "");
            match f(&mut d, i) {
                Ok(t) => { self.toml = d.toml; return Ok(t) }
                Err(e) => {
                    if first_error.is_none() {
                        first_error = Some(e);
                    }
                }
            }
        }
        Err(first_error.unwrap_or_else(|| self.err(NoEnumVariants)))
    }
    fn read_enum_variant_arg<T>(&mut self,
                                _a_idx: uint,
                                f: |&mut Decoder| -> Result<T, DecodeError>)
                                -> Result<T, DecodeError> {
        f(self)
    }

    fn read_enum_struct_variant<T>(&mut self,
                                   _names: &[&str],
                                   _f: |&mut Decoder, uint|
                                        -> Result<T, DecodeError>)
        -> Result<T, DecodeError>
    {
        fail!()
    }
    fn read_enum_struct_variant_field<T>(&mut self,
                                         _f_name: &str,
                                         _f_idx: uint,
                                         _f: |&mut Decoder|
                                            -> Result<T, DecodeError>)
        -> Result<T, DecodeError>
    {
        fail!()
    }

    fn read_struct<T>(&mut self, _s_name: &str, _len: uint,
                      f: |&mut Decoder| -> Result<T, DecodeError>)
        -> Result<T, DecodeError>
    {
        match self.toml {
            Some(Table(..)) => {
                let ret = try!(f(self));
                match self.toml {
                    Some(Table(ref t)) if t.len() == 0 => {}
                    _ => return Ok(ret)
                }
                self.toml.take();
                Ok(ret)
            }
            ref found => Err(self.mismatch("table", found)),
        }
    }
    fn read_struct_field<T>(&mut self,
                            f_name: &str,
                            _f_idx: uint,
                            f: |&mut Decoder| -> Result<T, DecodeError>)
                            -> Result<T, DecodeError> {
        let field = f_name.to_string();
        let toml = match self.toml {
            Some(Table(ref mut table)) => {
                table.pop(&field)
                    .or_else(|| table.pop(&hyphenate(f_name)))
            },
            ref found => return Err(self.mismatch("table", found)),
        };
        let mut d = self.sub_decoder(toml, f_name);
        let ret = try!(f(&mut d));
        match d.toml {
            Some(value) => match self.toml {
                Some(Table(ref mut table)) => { table.insert(field, value); }
                _ => {}
            },
            None => {}
        }
        Ok(ret)
    }

    fn read_tuple<T>(&mut self,
                     f: |&mut Decoder, uint| -> Result<T, DecodeError>)
        -> Result<T, DecodeError>
    {
        self.read_seq(f)
    }
    fn read_tuple_arg<T>(&mut self, a_idx: uint,
                         f: |&mut Decoder| -> Result<T, DecodeError>)
        -> Result<T, DecodeError>
    {
        self.read_seq_elt(a_idx, f)
    }

    fn read_tuple_struct<T>(&mut self,
                            _s_name: &str,
                            _f: |&mut Decoder, uint| -> Result<T, DecodeError>)
        -> Result<T, DecodeError>
    {
        fail!()
    }
    fn read_tuple_struct_arg<T>(&mut self,
                                _a_idx: uint,
                                _f: |&mut Decoder| -> Result<T, DecodeError>)
        -> Result<T, DecodeError>
    {
        fail!()
    }

    // Specialized types:
    fn read_option<T>(&mut self,
                      f: |&mut Decoder, bool| -> Result<T, DecodeError>)
        -> Result<T, DecodeError>
    {
        match self.toml {
            Some(..) => f(self, true),
            None => f(self, false),
        }
    }

    fn read_seq<T>(&mut self, f: |&mut Decoder, uint| -> Result<T, DecodeError>)
        -> Result<T, DecodeError>
    {
        let len = match self.toml {
            Some(Array(ref arr)) => arr.len(),
            ref found => return Err(self.mismatch("array", found)),
        };
        let ret = try!(f(self, len));
        match self.toml {
            Some(Array(ref mut arr)) => {
                arr.retain(|slot| slot.as_integer() != Some(0));
                if arr.len() != 0 { return Ok(ret) }
            }
            _ => return Ok(ret)
        }
        self.toml.take();
        Ok(ret)
    }
    fn read_seq_elt<T>(&mut self, idx: uint,
                       f: |&mut Decoder| -> Result<T, DecodeError>)
        -> Result<T, DecodeError>
    {
        let toml = match self.toml {
            Some(Array(ref mut arr)) => mem::replace(arr.get_mut(idx), Integer(0)),
            ref found => return Err(self.mismatch("array", found)),
        };
        let mut d = self.sub_decoder(Some(toml), "");
        let ret = try!(f(&mut d));
        match d.toml {
            Some(toml) => match self.toml {
                Some(Array(ref mut arr)) => *arr.get_mut(idx) = toml,
                _ => {}
            },
            _ => {}
        }
        Ok(ret)
    }

    fn read_map<T>(&mut self, f: |&mut Decoder, uint| -> Result<T, DecodeError>)
        -> Result<T, DecodeError>
    {
        let len = match self.toml {
            Some(Table(ref table)) => table.len(),
            ref found => return Err(self.mismatch("table", found)),
        };
        let ret = try!(f(self, len));
        self.toml.take();
        Ok(ret)
    }
    fn read_map_elt_key<T>(&mut self, idx: uint,
                           f: |&mut Decoder| -> Result<T, DecodeError>)
        -> Result<T, DecodeError>
    {
        match self.toml {
            Some(Table(ref table)) => {
                match table.keys().skip(idx).next() {
                    Some(key) => {
                        f(&mut self.sub_decoder(Some(String(key.to_string())),
                                                key.as_slice()))
                    }
                    None => Err(self.err(ExpectedMapKey(idx))),
                }
            }
            ref found => Err(self.mismatch("table", found)),
        }
    }
    fn read_map_elt_val<T>(&mut self, idx: uint,
                           f: |&mut Decoder| -> Result<T, DecodeError>)
        -> Result<T, DecodeError>
    {
        match self.toml {
            Some(Table(ref table)) => {
                match table.values().skip(idx).next() {
                    Some(key) => {
                        // XXX: this shouldn't clone
                        f(&mut self.sub_decoder(Some(key.clone()), ""))
                    }
                    None => Err(self.err(ExpectedMapElement(idx))),
                }
            }
            ref found => Err(self.mismatch("table", found)),
        }
    }
}

fn hyphenate(string: &str) -> String {
  str::replace(string, "_", "-")
}

impl fmt::Show for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(match self.kind {
            ExpectedField(expected_type) => {
                if expected_type == "table" {
                    write!(f, "expected a section")
                } else {
                    write!(f, "expected a value of type `{}`", expected_type)
                }
            }
            ExpectedType(expected, found) => {
                fn humanize(s: &str) -> String {
                    if s == "section" {
                        format!("a section")
                    } else {
                        format!("a value of type `{}`", s)
                    }
                }
                write!(f, "expected {}, but found {}",
                       humanize(expected),
                       humanize(found))
            }
            ExpectedMapKey(idx) => {
                write!(f, "expected at least {} keys", idx + 1)
            }
            ExpectedMapElement(idx) => {
                write!(f, "expected at least {} elements", idx + 1)
            }
            NoEnumVariants => {
                write!(f, "expected an enum variant to decode to")
            }
            NilTooLong => {
                write!(f, "expected 0-length string")
            }
        })
        match self.field {
            Some(ref s) => {
                write!(f, " for the key `{}`", s)
            }
            None => Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use serialize::{Encodable, Decodable};

    use super::{Encoder, Decoder, DecodeError};
    use {Table, Integer, String, Array, Float};

    macro_rules! encode( ($t:expr) => ({
        let mut e = Encoder::new();
        $t.encode(&mut e).unwrap();
        e.toml
    }) )

    macro_rules! decode( ($t:expr) => ({
        let mut d = Decoder::new($t);
        Decodable::decode(&mut d).unwrap()
    }) )

    macro_rules! map( ($($k:ident: $v:expr),*) => ({
        let mut _m = HashMap::new();
        $(_m.insert(stringify!($k).to_str(), $v);)*
        _m
    }) )

    #[test]
    fn smoke() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo { a: int }

        let v = Foo { a: 2 };
        assert_eq!(encode!(v), map! { a: Integer(2) });
        assert_eq!(v, decode!(Table(encode!(v))));
    }

    #[test]
    fn nested() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo { a: int, b: Bar }
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Bar { a: String }

        let v = Foo { a: 2, b: Bar { a: "test".to_string() } };
        assert_eq!(encode!(v),
                   map! {
                       a: Integer(2),
                       b: Table(map! {
                           a: String("test".to_string())
                       })
                   });
        assert_eq!(v, decode!(Table(encode!(v))));
    }

    #[test]
    fn array() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo { a: Vec<int> }

        let v = Foo { a: vec![1, 2, 3, 4] };
        assert_eq!(encode!(v),
                   map! {
                       a: Array(vec![
                            Integer(1),
                            Integer(2),
                            Integer(3),
                            Integer(4)
                       ])
                   });
        assert_eq!(v, decode!(Table(encode!(v))));
    }

    #[test]
    fn tuple() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo { a: (int, int, int, int) }

        let v = Foo { a: (1, 2, 3, 4) };
        assert_eq!(encode!(v),
                   map! {
                       a: Array(vec![
                            Integer(1),
                            Integer(2),
                            Integer(3),
                            Integer(4)
                       ])
                   });
        assert_eq!(v, decode!(Table(encode!(v))));
    }

    #[test]
    fn inner_structs_with_options() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo {
            a: Option<Box<Foo>>,
            b: Bar,
        }
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Bar {
            a: String,
            b: f64,
        }

        let v = Foo {
            a: Some(box Foo {
                a: None,
                b: Bar { a: "foo".to_string(), b: 4.5 },
            }),
            b: Bar { a: "bar".to_string(), b: 1.0 },
        };
        assert_eq!(encode!(v),
                   map! {
                       a: Table(map! {
                           b: Table(map! {
                               a: String("foo".to_string()),
                               b: Float(4.5)
                           })
                       }),
                       b: Table(map! {
                           a: String("bar".to_string()),
                           b: Float(1.0)
                       })
                   });
        assert_eq!(v, decode!(Table(encode!(v))));
    }

    #[test]
    fn hashmap() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo {
            map: HashMap<String, int>,
            set: HashSet<char>,
        }

        let v = Foo {
            map: {
                let mut m = HashMap::new();
                m.insert("foo".to_string(), 10);
                m.insert("bar".to_string(), 4);
                m
            },
            set: {
                let mut s = HashSet::new();
                s.insert('a');
                s
            },
        };
        assert_eq!(encode!(v),
            map! {
                map: Table(map! {
                    foo: Integer(10),
                    bar: Integer(4)
                }),
                set: Array(vec![String("a".to_str())])
            }
        );
        assert_eq!(v, decode!(Table(encode!(v))));
    }

    #[test]
    fn tuple_struct() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo(int, String, f64);

        let v = Foo(1, "foo".to_string(), 4.5);
        assert_eq!(
            encode!(v),
            map! {
                _field0: Integer(1),
                _field1: String("foo".to_string()),
                _field2: Float(4.5)
            }
        );
        assert_eq!(v, decode!(Table(encode!(v))));
    }

    #[test]
    fn table_array() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo { a: Vec<Bar>, }
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Bar { a: int }

        let v = Foo { a: vec![Bar { a: 1 }, Bar { a: 2 }] };
        assert_eq!(
            encode!(v),
            map! {
                a: Array(vec![
                    Table(map!{ a: Integer(1) }),
                    Table(map!{ a: Integer(2) }),
                ])
            }
        );
        assert_eq!(v, decode!(Table(encode!(v))));
    }

    #[test]
    fn type_errors() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo { bar: int }

        let mut d = Decoder::new(Table(map! {
            bar: Float(1.0)
        }));
        let a: Result<Foo, DecodeError> = Decodable::decode(&mut d);
        match a {
            Ok(..) => fail!("should not have decoded"),
            Err(e) => {
                assert_eq!(e.to_str().as_slice(),
                           "expected a value of type `integer`, but \
                            found a value of type `float` for the key `bar`");
            }
        }
    }

    #[test]
    fn missing_errors() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo { bar: int }

        let mut d = Decoder::new(Table(map! {
        }));
        let a: Result<Foo, DecodeError> = Decodable::decode(&mut d);
        match a {
            Ok(..) => fail!("should not have decoded"),
            Err(e) => {
                assert_eq!(e.to_str().as_slice(),
                           "expected a value of type `integer` for the key `bar`");
            }
        }
    }

    #[test]
    fn parse_enum() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo { a: E }
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        enum E {
            Bar(int),
            Baz(f64),
            Last(Foo2),
        }
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo2 {
            test: String,
        }

        let v = Foo { a: Bar(10) };
        assert_eq!(
            encode!(v),
            map! { a: Integer(10) }
        );
        assert_eq!(v, decode!(Table(encode!(v))));

        let v = Foo { a: Baz(10.2) };
        assert_eq!(
            encode!(v),
            map! { a: Float(10.2) }
        );
        assert_eq!(v, decode!(Table(encode!(v))));

        let v = Foo { a: Last(Foo2 { test: "test".to_string() }) };
        assert_eq!(
            encode!(v),
            map! { a: Table(map! { test: String("test".to_string()) }) }
        );
        assert_eq!(v, decode!(Table(encode!(v))));
    }

    #[test]
    fn unused_fields() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo { a: int }

        let v = Foo { a: 2 };
        let mut d = Decoder::new(Table(map! {
            a: Integer(2),
            b: Integer(5)
        }));
        assert_eq!(v, Decodable::decode(&mut d).unwrap());

        assert_eq!(d.toml, Some(Table(map! {
            b: Integer(5)
        })));
    }

    #[test]
    fn unused_fields2() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo { a: Bar }
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Bar { a: int }

        let v = Foo { a: Bar { a: 2 } };
        let mut d = Decoder::new(Table(map! {
            a: Table(map! {
                a: Integer(2),
                b: Integer(5)
            })
        }));
        assert_eq!(v, Decodable::decode(&mut d).unwrap());

        assert_eq!(d.toml, Some(Table(map! {
            a: Table(map! {
                b: Integer(5)
            })
        })));
    }

    #[test]
    fn unused_fields3() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo { a: Bar }
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Bar { a: int }

        let v = Foo { a: Bar { a: 2 } };
        let mut d = Decoder::new(Table(map! {
            a: Table(map! {
                a: Integer(2)
            })
        }));
        assert_eq!(v, Decodable::decode(&mut d).unwrap());

        assert_eq!(d.toml, None);
    }

    #[test]
    fn unused_fields4() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo { a: HashMap<String, String> }

        let v = Foo { a: map! { a: "foo".to_string() } };
        let mut d = Decoder::new(Table(map! {
            a: Table(map! {
                a: String("foo".to_string())
            })
        }));
        assert_eq!(v, Decodable::decode(&mut d).unwrap());

        assert_eq!(d.toml, None);
    }

    #[test]
    fn unused_fields5() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo { a: Vec<String> }

        let v = Foo { a: vec!["a".to_string()] };
        let mut d = Decoder::new(Table(map! {
            a: Array(vec![String("a".to_string())])
        }));
        assert_eq!(v, Decodable::decode(&mut d).unwrap());

        assert_eq!(d.toml, None);
    }

    #[test]
    fn unused_fields6() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo { a: Option<Vec<String>> }

        let v = Foo { a: Some(vec![]) };
        let mut d = Decoder::new(Table(map! {
            a: Array(vec![])
        }));
        assert_eq!(v, Decodable::decode(&mut d).unwrap());

        assert_eq!(d.toml, None);
    }

    #[test]
    fn unused_fields7() {
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Foo { a: Vec<Bar> }
        #[deriving(Encodable, Decodable, PartialEq, Show)]
        struct Bar { a: int }

        let v = Foo { a: vec![Bar { a: 1 }] };
        let mut d = Decoder::new(Table(map! {
            a: Array(vec![Table(map! {
                a: Integer(1),
                b: Integer(2)
            })])
        }));
        assert_eq!(v, Decodable::decode(&mut d).unwrap());

        assert_eq!(d.toml, Some(Table(map! {
            a: Array(vec![Table(map! {
                b: Integer(2)
            })])
        })));
    }
}

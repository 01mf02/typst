use std::any::Any;
use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use super::{ops, Args, Array, Cast, Dict, Func, RawLength, Str};
use crate::diag::StrResult;
use crate::geom::{Angle, Color, Em, Fraction, Length, Ratio, Relative, RgbaColor};
use crate::library::text::RawNode;
use crate::model::{Content, Layout, Transform};
use crate::util::EcoString;

/// A computational value.
#[derive(Clone)]
pub enum Value {
    /// The value that indicates the absence of a meaningful value.
    None,
    /// A value that indicates some smart default behaviour.
    Auto,
    /// A boolean: `true, false`.
    Bool(bool),
    /// An integer: `120`.
    Int(i64),
    /// A floating-point number: `1.2`, `10e-4`.
    Float(f64),
    /// A length: `12pt`, `3cm`, `1.5em`.
    Length(RawLength),
    /// An angle: `1.5rad`, `90deg`.
    Angle(Angle),
    /// A ratio: `50%`.
    Ratio(Ratio),
    /// A relative length, combination of a ratio and a length: `20% + 5cm`.
    Relative(Relative<RawLength>),
    /// A fraction: `1fr`.
    Fraction(Fraction),
    /// A color value: `#f79143ff`.
    Color(Color),
    /// A string: `"string"`.
    Str(Str),
    /// A content value: `[*Hi* there]`.
    Content(Content),
    /// A transform value: `set text(fill: blue)`.
    Transform(Transform),
    /// An array of values: `(1, "hi", 12cm)`.
    Array(Array),
    /// A dictionary value: `(color: #f79143, pattern: dashed)`.
    Dict(Dict),
    /// An executable function.
    Func(Func),
    /// Captured arguments to a function.
    Args(Args),
    /// A dynamic value.
    Dyn(Dynamic),
}

impl Value {
    /// Create a content value from an inline-level node.
    pub fn inline<T>(node: T) -> Self
    where
        T: Layout + Debug + Hash + Sync + Send + 'static,
    {
        Self::Content(Content::inline(node))
    }

    /// Create a content value from a block-level node.
    pub fn block<T>(node: T) -> Self
    where
        T: Layout + Debug + Hash + Sync + Send + 'static,
    {
        Self::Content(Content::block(node))
    }

    /// Create a new dynamic value.
    pub fn dynamic<T>(any: T) -> Self
    where
        T: Type + Debug + PartialEq + Hash + Sync + Send + 'static,
    {
        Self::Dyn(Dynamic::new(any))
    }

    /// The name of the stored value's type.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Auto => "auto",
            Self::Bool(_) => bool::TYPE_NAME,
            Self::Int(_) => i64::TYPE_NAME,
            Self::Float(_) => f64::TYPE_NAME,
            Self::Length(_) => RawLength::TYPE_NAME,
            Self::Angle(_) => Angle::TYPE_NAME,
            Self::Ratio(_) => Ratio::TYPE_NAME,
            Self::Relative(_) => Relative::<RawLength>::TYPE_NAME,
            Self::Fraction(_) => Fraction::TYPE_NAME,
            Self::Color(_) => Color::TYPE_NAME,
            Self::Str(_) => Str::TYPE_NAME,
            Self::Content(_) => Content::TYPE_NAME,
            Self::Transform(_) => Transform::TYPE_NAME,
            Self::Array(_) => Array::TYPE_NAME,
            Self::Dict(_) => Dict::TYPE_NAME,
            Self::Func(_) => Func::TYPE_NAME,
            Self::Args(_) => Args::TYPE_NAME,
            Self::Dyn(v) => v.type_name(),
        }
    }

    /// Try to cast the value into a specific type.
    pub fn cast<T: Cast>(self) -> StrResult<T> {
        T::cast(self)
    }

    /// Return the debug representation of the value.
    pub fn repr(&self) -> Str {
        format_str!("{:?}", self)
    }

    /// Return the display representation of the value.
    pub fn display(self) -> Content {
        match self {
            Value::None => Content::new(),
            Value::Int(v) => Content::Text(format_eco!("{}", v)),
            Value::Float(v) => Content::Text(format_eco!("{}", v)),
            Value::Str(v) => Content::Text(v.into()),
            Value::Content(v) => v,

            // For values which can't be shown "naturally", we return the raw
            // representation with typst code syntax highlighting.
            v => Content::show(RawNode { text: v.repr().into(), block: false })
                .styled(RawNode::LANG, Some("typc".into())),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::None
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::None => f.pad("none"),
            Self::Auto => f.pad("auto"),
            Self::Bool(v) => Debug::fmt(v, f),
            Self::Int(v) => Debug::fmt(v, f),
            Self::Float(v) => Debug::fmt(v, f),
            Self::Length(v) => Debug::fmt(v, f),
            Self::Angle(v) => Debug::fmt(v, f),
            Self::Ratio(v) => Debug::fmt(v, f),
            Self::Relative(v) => Debug::fmt(v, f),
            Self::Fraction(v) => Debug::fmt(v, f),
            Self::Color(v) => Debug::fmt(v, f),
            Self::Str(v) => Debug::fmt(v, f),
            Self::Content(_) => f.pad("[...]"),
            Self::Transform(_) => f.pad("<transform>"),
            Self::Array(v) => Debug::fmt(v, f),
            Self::Dict(v) => Debug::fmt(v, f),
            Self::Func(v) => Debug::fmt(v, f),
            Self::Args(v) => Debug::fmt(v, f),
            Self::Dyn(v) => Debug::fmt(v, f),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        ops::equal(self, other)
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        ops::compare(self, other)
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::None => {}
            Self::Auto => {}
            Self::Bool(v) => v.hash(state),
            Self::Int(v) => v.hash(state),
            Self::Float(v) => v.to_bits().hash(state),
            Self::Length(v) => v.hash(state),
            Self::Angle(v) => v.hash(state),
            Self::Ratio(v) => v.hash(state),
            Self::Relative(v) => v.hash(state),
            Self::Fraction(v) => v.hash(state),
            Self::Color(v) => v.hash(state),
            Self::Str(v) => v.hash(state),
            Self::Content(v) => v.hash(state),
            Self::Transform(v) => v.hash(state),
            Self::Array(v) => v.hash(state),
            Self::Dict(v) => v.hash(state),
            Self::Func(v) => v.hash(state),
            Self::Args(v) => v.hash(state),
            Self::Dyn(v) => v.hash(state),
        }
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Self::Int(v as i64)
    }
}

impl From<usize> for Value {
    fn from(v: usize) -> Self {
        Self::Int(v as i64)
    }
}

impl From<Length> for Value {
    fn from(v: Length) -> Self {
        Self::Length(v.into())
    }
}

impl From<Em> for Value {
    fn from(v: Em) -> Self {
        Self::Length(v.into())
    }
}

impl From<RgbaColor> for Value {
    fn from(v: RgbaColor) -> Self {
        Self::Color(v.into())
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::Str(v.into())
    }
}

impl From<EcoString> for Value {
    fn from(v: EcoString) -> Self {
        Self::Str(v.into())
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::Str(v.into())
    }
}

impl From<Dynamic> for Value {
    fn from(v: Dynamic) -> Self {
        Self::Dyn(v)
    }
}

/// A dynamic value.
#[derive(Clone, Hash)]
pub struct Dynamic(Arc<dyn Bounds>);

impl Dynamic {
    /// Create a new instance from any value that satisifies the required bounds.
    pub fn new<T>(any: T) -> Self
    where
        T: Type + Debug + PartialEq + Hash + Sync + Send + 'static,
    {
        Self(Arc::new(any))
    }

    /// Whether the wrapped type is `T`.
    pub fn is<T: Type + 'static>(&self) -> bool {
        (*self.0).as_any().is::<T>()
    }

    /// Try to downcast to a reference to a specific type.
    pub fn downcast<T: Type + 'static>(&self) -> Option<&T> {
        (*self.0).as_any().downcast_ref()
    }

    /// The name of the stored value's type.
    pub fn type_name(&self) -> &'static str {
        self.0.dyn_type_name()
    }
}

impl Debug for Dynamic {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl PartialEq for Dynamic {
    fn eq(&self, other: &Self) -> bool {
        self.0.dyn_eq(other)
    }
}

trait Bounds: Debug + Sync + Send + 'static {
    fn as_any(&self) -> &dyn Any;
    fn dyn_eq(&self, other: &Dynamic) -> bool;
    fn dyn_type_name(&self) -> &'static str;
    fn hash64(&self) -> u64;
}

impl<T> Bounds for T
where
    T: Type + Debug + PartialEq + Hash + Sync + Send + 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn dyn_eq(&self, other: &Dynamic) -> bool {
        if let Some(other) = other.downcast::<Self>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_type_name(&self) -> &'static str {
        T::TYPE_NAME
    }

    fn hash64(&self) -> u64 {
        // Also hash the TypeId since nodes with different types but
        // equal data should be different.
        let mut state = fxhash::FxHasher64::default();
        self.type_id().hash(&mut state);
        self.hash(&mut state);
        state.finish()
    }
}

impl Hash for dyn Bounds {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(self.hash64());
    }
}

/// The type of a value.
pub trait Type {
    /// The name of the type.
    const TYPE_NAME: &'static str;
}

/// Implement traits for primitives.
macro_rules! primitive {
    (
        $type:ty: $name:literal, $variant:ident
        $(, $other:ident$(($binding:ident))? => $out:expr)*
    ) => {
        impl Type for $type {
            const TYPE_NAME: &'static str = $name;
        }

        impl Cast for $type {
            fn is(value: &Value) -> bool {
                matches!(value, Value::$variant(_)
                    $(|  primitive!(@$other $(($binding))?))*)
            }

            fn cast(value: Value) -> StrResult<Self> {
                match value {
                    Value::$variant(v) => Ok(v),
                    $(Value::$other$(($binding))? => Ok($out),)*
                    v => Err(format!(
                        "expected {}, found {}",
                        Self::TYPE_NAME,
                        v.type_name(),
                    )),
                }
            }
        }

        impl From<$type> for Value {
            fn from(v: $type) -> Self {
                Value::$variant(v)
            }
        }
    };

    (@$other:ident($binding:ident)) => { Value::$other(_) };
    (@$other:ident) => { Value::$other };
}

primitive! { bool: "boolean", Bool }
primitive! { i64: "integer", Int }
primitive! { f64: "float", Float, Int(v) => v as f64 }
primitive! { RawLength: "length", Length }
primitive! { Angle: "angle", Angle }
primitive! { Ratio: "ratio", Ratio }
primitive! { Relative<RawLength>:  "relative length",
    Relative,
    Length(v) => v.into(),
    Ratio(v) => v.into()
}
primitive! { Fraction: "fraction", Fraction }
primitive! { Color: "color", Color }
primitive! { Str: "string", Str }
primitive! { Content: "content",
    Content,
    None => Content::new(),
    Str(text) => Content::Text(text.into())
}
primitive! { Transform: "transform", Transform }
primitive! { Array: "array", Array }
primitive! { Dict: "dictionary", Dict }
primitive! { Func: "function", Func }
primitive! { Args: "arguments", Args }

#[cfg(test)]
mod tests {
    use super::*;

    #[track_caller]
    fn test(value: impl Into<Value>, exp: &str) {
        assert_eq!(format!("{:?}", value.into()), exp);
    }

    #[test]
    fn test_value_debug() {
        // Primitives.
        test(Value::None, "none");
        test(false, "false");
        test(12i64, "12");
        test(3.14, "3.14");
        test(Length::pt(5.5), "5.5pt");
        test(Angle::deg(90.0), "90deg");
        test(Ratio::one() / 2.0, "50%");
        test(
            Ratio::new(0.3) + RawLength::from(Length::cm(2.0)),
            "30% + 56.69pt",
        );
        test(Fraction::one() * 7.55, "7.55fr");
        test(
            Color::Rgba(RgbaColor::new(1, 1, 1, 0xff)),
            "rgb(\"#010101\")",
        );

        // Collections.
        test("hello", r#""hello""#);
        test("\n", r#""\n""#);
        test("\\", r#""\\""#);
        test("\"", r#""\"""#);
        test(array![], "()");
        test(array![Value::None], "(none,)");
        test(array![1, 2], "(1, 2)");
        test(dict![], "(:)");
        test(dict!["one" => 1], "(one: 1)");
        test(dict!["two" => false, "one" => 1], "(one: 1, two: false)");

        // Functions, content and dynamics.
        test(Content::Text("a".into()), "[...]");
        test(Func::from_fn("nil", |_, _| Ok(Value::None)), "nil");
        test(Dynamic::new(1), "1");
    }
}

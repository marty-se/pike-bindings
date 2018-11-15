use ::pike::*;
use ::pike::pike_svalue::svalue;
use ::serde::ser::*;
use ::serde::*;

use std::fmt;

use serde::de::{Visitor, MapAccess, SeqAccess};

pub trait Refcounted<TPtr>: Drop + CloneWithCtx {
    unsafe fn from_ptr<'ctx>(ptr: *mut TPtr) -> Self;
    unsafe fn from_ptr_add_ref<'ctx>(ptr: *mut TPtr, ctx: &'ctx PikeContext) -> Self;
    fn as_mut_ptr(&self) -> *mut TPtr;
}

pub trait CloneWithCtx: Sized {
    fn clone_with_ctx<'ctx>(&self, ctx: &'ctx PikeContext) -> Self;
}

pub trait FromWithCtx<'ctx, T>: Sized {
    fn from_with_ctx(_: T, ctx: &'ctx PikeContext) -> Self;
}

pub trait IntoWithCtx<'ctx, T>: Sized {
    fn into_with_ctx(self, ctx: &'ctx PikeContext) -> T;
}

impl<'ctx, T, U> IntoWithCtx<'ctx, U> for T where U: FromWithCtx<'ctx, T>
{
    fn into_with_ctx(self, ctx: &'ctx PikeContext) -> U {
        U::from_with_ctx(self, ctx)
    }
}

impl<'ctx, T, U> FromWithCtx<'ctx, T> for U where U: From<T>
{
    fn from_with_ctx(val: T, _ctx: &'ctx PikeContext) -> Self {
        Self::from(val)
    }
}

/// The `PikeThing` type. Equivalent to Pike's `svalue` type, with Rust idioms.
#[derive(Debug)]
pub enum PikeThing {
    Array(PikeArrayRef),
    Float(PikeFloat),
    Function(PikeFunctionRef),
    Int(PikeInt),
    Mapping(PikeMappingRef),
    Multiset(PikeMultisetRef),
    Object(PikeObjectRef<()>),
    PikeString(PikeStringRef),
    Program(PikeProgramRef<()>),
    Type(PikeTypeRef),
    Undefined
}

#[derive(Debug)]
pub struct PikeThingWithCtx<'ctx> {
    thing: PikeThing,
    ctx: &'ctx PikeContext
}

impl PikeThing {
    pub fn from_svalue_ref(sval: &svalue, ctx: &PikeContext) -> Self {
        let new_sval: svalue = sval.clone(ctx);
        new_sval.into()
    }

    /// Instantiates a PikeThing representing Pike's UNDEFINED value.
    pub fn undefined() -> Self {
        let sval = svalue::undefined();
        let res: PikeThing = sval.into();
        return res;
    }

    pub fn clone_with_ctx(&self, ctx: &PikeContext) -> PikeThing {
        match self {
            PikeThing::Array(a) => PikeThing::Array(a.clone_with_ctx(ctx)),
            PikeThing::Float(f) => PikeThing::Float(f.clone()),
            PikeThing::Function(f) => PikeThing::Function(f.clone_with_ctx(ctx)),
            PikeThing::Int(i) => PikeThing::Int(i.clone()),
            PikeThing::Mapping(m) => PikeThing::Mapping(m.clone_with_ctx(ctx)),
            PikeThing::Multiset(m) => PikeThing::Multiset(m.clone_with_ctx(ctx)),
            PikeThing::Object(o) => PikeThing::Object(o.clone_with_ctx(ctx)),
            PikeThing::PikeString(s) => PikeThing::PikeString(s.clone_with_ctx(ctx)),
            PikeThing::Program(p) => PikeThing::Program(p.clone_with_ctx(ctx)),
            PikeThing::Type(t) => PikeThing::Type(t.clone_with_ctx(ctx)),
            PikeThing::Undefined => PikeThing::Undefined
        }
    }

    pub fn unwrap<'ctx>(self, ctx: &'ctx PikeContext) -> PikeThingWithCtx<'ctx> {
        PikeThingWithCtx { thing: self, ctx: ctx }
    }
}

impl<'ctx> Serialize for PikeThingWithCtx<'ctx> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: ::serde::Serializer {
        let ctx = self.ctx;
        match self.thing {
            PikeThing::Array(ref a_ref) => {
                let a: PikeArray = a_ref.clone_with_ctx(ctx).into_with_ctx(ctx);
                a.serialize(serializer)
            }
            PikeThing::Mapping(ref m_ref) => {
                let m: PikeMapping = m_ref.clone_with_ctx(ctx).into_with_ctx(ctx);
                m.serialize(serializer)
            }
            PikeThing::Multiset(ref m_ref) => {
                let m: PikeMultiset = m_ref.clone_with_ctx(ctx).into_with_ctx(ctx);
                m.serialize(serializer)
            }
            PikeThing::PikeString(ref s_ref) => {
                let s: PikeString = s_ref.clone_with_ctx(ctx).into_with_ctx(ctx);
                s.serialize(serializer)
            }
            PikeThing::Int(ref i) => {
                i.serialize(serializer)
            }
            PikeThing::Float(ref f) => {
                f.serialize(serializer)
            }
            _ => Err(ser::Error::custom("Unsupported Pike type"))
    }
  }
}

struct PikeThingVisitor<'ctx> {
    ctx: &'ctx PikeContext
}

impl<'de, 'ctx> Visitor<'de> for PikeThingVisitor<'ctx> {
    type Value = PikeThing;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("something serializeable to a Pike thing")
    }

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E> {
        Ok(value.into())
    }

    fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E> {
        Ok(value.into())
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E> {
        Ok(value.into())
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(value.into())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
        let pike_str = PikeString::from_str_slice(v, self.ctx);
        Ok(PikeThing::PikeString(pike_str.into()))
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where M: MapAccess<'de>,
    {
        let m = PikeMapping::with_capacity(access.size_hint().unwrap_or(0),
            self.ctx);

        // While there are entries remaining in the input, add them
        // into our map.
        while let Some((key, value)) = access.next_entry()? {
            m.insert(key, value);
        }

        Ok(PikeThing::Mapping(m.into()))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where A: SeqAccess<'de>,
    {
        let mut a =
            PikeArray::with_capacity(seq.size_hint().unwrap_or(0), self.ctx);

        while let Some(value) = seq.next_element()? {
            a.append(value)
        }
        Ok(PikeThing::Array(a.into()))
    }
}

impl<'de> Deserialize<'de> for PikeThing {
    fn deserialize<D>(deserializer: D) -> Result<PikeThing, D::Error>
    where D: Deserializer<'de> {
        PikeContext::call_with_context(|ctx| {
            deserializer.deserialize_any(PikeThingVisitor { ctx: &ctx })
        })
  }
}

impl From<()> for PikeThing {
  fn from(_: ()) -> PikeThing {
    return PikeThing::undefined();
  }
}

macro_rules! gen_from_type_int {
  ($inttype: ident) => {
    impl From<$inttype> for PikeThing {
      fn from(i: $inttype) -> PikeThing {
      return PikeThing::Int(i.into());
      }
    }
  };
}

gen_from_type_int!(u64);
gen_from_type_int!(u32);
gen_from_type_int!(u16);
gen_from_type_int!(u8);

gen_from_type_int!(i64);
gen_from_type_int!(i32);
gen_from_type_int!(i16);
gen_from_type_int!(i8);

macro_rules! gen_from_type_float {
  ($floattype: ident) => {
    impl From<$floattype> for PikeThing {
      fn from(f: $floattype) -> PikeThing {
      return PikeThing::Float(f.into());
      }
    }
  };
}

gen_from_type_float!(f64);
gen_from_type_float!(f32);

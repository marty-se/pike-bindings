use ::types::type_deps::*;
use ::interpreter::PikeContext;

pub use ::ffi::svalue;
use ::ffi::{PIKE_T_FLOAT, PIKE_T_INT, PIKE_T_ARRAY, PIKE_T_FUNCTION,
    PIKE_T_MAPPING, PIKE_T_MULTISET, PIKE_T_OBJECT, PIKE_T_PROGRAM, PIKE_T_TYPE,
	PIKE_T_FREE, PIKE_T_STRING, NUMBER_UNDEFINED};

impl From<svalue> for PikeThing {
    fn from (sval: svalue) -> Self {
        let type_ = unsafe { sval.tu.t.type_ };
        let subtype = unsafe { sval.tu.t.subtype };
        let res = match type_.into() {
            PIKE_T_ARRAY => {
                PikeThing::Array(
                    unsafe {
                        PikeArrayRef::from_ptr(sval.u.array)
                    })
            },
            PIKE_T_FLOAT => {
                PikeThing::Float(PikeFloat::new(
                    unsafe { sval.u.float_number }))
            },
            PIKE_T_FUNCTION => {
                PikeThing::Function(
                    unsafe {
                        PikeFunctionRef::new_without_ref(sval.u.object, subtype)
                    })
			},
			PIKE_T_INT => {
                if subtype == NUMBER_UNDEFINED as u16 {
                    PikeThing::Undefined
                } else {
                    PikeThing::Int(PikeInt::new(
                        unsafe { sval.u.integer }))
                }
            },
            PIKE_T_MAPPING => {
                PikeThing::Mapping(
					unsafe {
                        PikeMappingRef::from_ptr(sval.u.mapping)
                    })
            },
            PIKE_T_MULTISET => {
                PikeThing::Multiset(
					unsafe {
                        PikeMultisetRef::from_ptr(sval.u.multiset)
                    })
            },
            PIKE_T_OBJECT => {
                PikeThing::Object(
					unsafe {
                        PikeObjectRef::<()>::from_ptr(sval.u.object)
                    })
            },
            PIKE_T_STRING => {
                PikeThing::PikeString(
					unsafe {
                        PikeStringRef::from_ptr(sval.u.string)
                    })
            },
            PIKE_T_PROGRAM => {
                PikeThing::Program(
					unsafe {
                        PikeProgramRef::<()>::from_ptr(sval.u.program)
                    })
            },
            PIKE_T_TYPE => {
                PikeThing::Type(unsafe { PikeTypeRef::from_ptr(sval.u.type_ )})
            },
            _ => panic!("Unknown Pike type.")
        };

		// Reference is transferred - forget sval to avoid calling Drop
		// destructor. This allows this code to be called without a PikeContext
		// -- i.e. without holding the Pike interpreter lock.
        ::std::mem::forget(sval);
        res
    }
}

impl From<PikeThing> for svalue {
    fn from (pike_thing: PikeThing) -> Self {
        let mut u = ::ffi::anything { integer: 0 };
        let type_: u32;
		let mut subtype: u16 = 0;

        match pike_thing {
            PikeThing::Array(ref a) => {
				u.array = a.as_mut_ptr();
				type_ = PIKE_T_ARRAY;
            }
            PikeThing::Float(ref f) => {
                u.float_number = f.into();
				type_ = PIKE_T_FLOAT;
            }
            PikeThing::Function(ref f) => {
                u.object = f.object_ptr();
				type_ = PIKE_T_FUNCTION;
				subtype = f.function_index();
            }
            PikeThing::Int(ref i) => {
				u.integer = i.into();
                type_ = PIKE_T_INT;
            }
            PikeThing::Mapping(ref m) => {
                u.mapping = m.as_mut_ptr();
				type_ = PIKE_T_MAPPING;
            }
            PikeThing::Multiset(ref m) => {
                u.multiset = m.as_mut_ptr();
				type_ = PIKE_T_MULTISET;
            }
            PikeThing::Object(ref o) => {
                u.object = o.as_mut_ptr();
				type_ = PIKE_T_OBJECT;
            }
            PikeThing::PikeString(ref s) => {
                u.string = s.as_mut_ptr();
				type_ = PIKE_T_STRING;
            }
            PikeThing::Program(ref p) => {
                u.program = p.as_mut_ptr();
				type_ = PIKE_T_PROGRAM;
            }
            PikeThing::Type(ref t) => {
                u.type_ = t.as_mut_ptr();
				type_ = PIKE_T_TYPE;
            }
            PikeThing::Undefined => {
                return svalue::undefined();
            }
        }

        // The ref is transferred, so we don't want to run the destructor (for
		// reference types).
		// This enables ref transferring without holding Pike's interpreter lock.
		::std::mem::forget(pike_thing);

        let t = ::ffi::svalue__bindgen_ty_1__bindgen_ty_1 {
            type_: type_ as ::std::os::raw::c_ushort,
            subtype: subtype };
        let tu = ::ffi::svalue__bindgen_ty_1 {t: t};
        ::ffi::svalue {u: u, tu: tu}
    }
}

impl ::std::default::Default for svalue {
    fn default() -> Self {
        svalue::undefined()
    }
}

impl svalue {
    pub fn undefined() -> Self {
        let a = ::ffi::anything { integer: 0 };
        let t = ::ffi::svalue__bindgen_ty_1__bindgen_ty_1 {
            type_: PIKE_T_INT as ::std::os::raw::c_ushort,
            subtype: NUMBER_UNDEFINED as u16 };
        let tu = ::ffi::svalue__bindgen_ty_1 {t: t};
        ::ffi::svalue {u: a, tu: tu}
    }

    pub fn clone(&self, ctx: &PikeContext) -> Self {
        let t = unsafe { ::ffi::svalue__bindgen_ty_1__bindgen_ty_1 {
            type_: self.tu.t.type_,
            subtype: self.tu.t.subtype }
        };
        let tu = ::ffi::svalue__bindgen_ty_1 {t: t};

        let u = ::ffi::anything { array: unsafe { self.u.array }};

        let mut res = svalue { u: u, tu: tu };
        res.add_ref(ctx);
        res
    }

    pub fn add_ref(&mut self, _ctx: &PikeContext) -> Option<usize> {
        if self.refcounted_type() {
            unsafe {
                let r = self.u.dummy;
                (*r).refs += 1;
                return Some((*r).refs as usize);
            }
        }
        None
    }

    pub fn sub_ref(&mut self, _ctx: &PikeContext) -> Option<usize> {
        if self.refcounted_type() {
            unsafe {
                let r = self.u.dummy;
                (*r).refs -= 1;
                return Some((*r).refs as usize);
            }
        }
        None
    }

    pub fn mark_free(&mut self) {
        unsafe {
            self.tu.t.type_ = PIKE_T_FREE as u16;
        }
    }

    fn type_(&self) -> u16 {
        unsafe {
            self.tu.t.type_
        }
    }

    #[allow(dead_code)]
    fn subtype(&self) -> u16 {
        unsafe { self.tu.t.subtype }
    }

    fn refcounted_type(&self) -> bool {
        // Equivalent of REFCOUNTED_TYPE macro in svalue.h
        (self.type_() & !(PIKE_T_ARRAY as u16 - 1)) != 0
    }
}

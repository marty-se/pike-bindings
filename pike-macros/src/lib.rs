#![recursion_limit="128"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use proc_macro::TokenStream;

#[derive(Clone, Serialize, Deserialize, Debug)]
enum TypeMetadata {
    Int,
    Float,
    Custom {
        name: Option< String >,
        conversion_fn: String
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct ArgMetadata {
    name: String,
    ty: TypeMetadata
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct ExportMetadata {
    name: String,
    args: Vec< ArgMetadata >,
    result: Option< TypeMetadata >
}

enum ExportType {
    SelfRef,
    Unit,
    Int,
    Float,
    StrRef,
    String,
    Fn,
    PikeFunction,
    PikeThing,
    PikeString,
    Result(Box<ExportType>),
    Wrapped(syn::Type),
    WrappedRef(syn::Type),
    Slice(syn::Type),
    Unknown(syn::Type)
}

struct ExportArg {
    ident: syn::Ident,
    ty: ExportType
}

struct Export {
    ident: syn::Ident,
    return_ty: ExportType,
    args: Vec<ExportArg>,
    is_constructor: bool,
    struct_ty: Option<syn::Ident>
}

fn match_shallow_path( path: &syn::Path ) -> Option< &str > {
    //let segs: &Vec<&str> = &path.segments.iter().map(|s| s.ident.as_ref()).collect();

    if path.leading_colon.is_some() || path.segments.len() != 1 {
        return None;
    }

    let segment = &path.segments[ 0 ];
    let name = segment.ident.as_ref();
    match &segment.arguments {
        &syn::PathArguments::None => {
            Some( name )
        },
        &syn::PathArguments::Parenthesized(ref _p) => {
            match name {
                "Fn" => Some(name),
                _ => None
            }
        }
        _ => { None }
    }
}

fn match_result_type(ty: &syn::Type) -> Option<ExportType> {
    if let &syn::Type::Path(ref type_path) = ty {
        if let Some(last_seg) = &type_path.path.segments.last() {
            let seg = last_seg.value();
            if seg.ident == "Result" {
                match seg.arguments {
                    syn::PathArguments::AngleBracketed(ref abga) => {
                        match abga.args.first() {
                            Some(first_arg) => {
                                match first_arg.into_value() {
                                    syn::GenericArgument::Type(gty) => {
                                        let inner_type = match_type (&gty);
                                        let res = ExportType::Result(
                                            Box::new(inner_type));
                                        return Some(res);
                                    },
                                    _ => {}
                                }
                            },
                            None => {}
                        }
                    },
                    _ => {}
                }
            }
        }
    }
    None
}

// Returns the ident of the last segment of the type's path, if any.
fn ident_from_type(ty: &syn::Type) -> Option<syn::Ident> {
    match ty {
        &syn::Type::Path(ref type_path) => {
            if let Some(last_seg) = &type_path.path.segments.last() {
                let seg = last_seg.value();
                return Some(seg.ident.clone());
            }
        },
        _ => {}
    }
    None
}

fn match_type( ty: &syn::Type ) -> ExportType {
    match ty {
        &syn::Type::Reference( ref ty ) => {
            assert!( ty.mutability.is_none(), "`mut` bindings are not supported" );
            match *ty.elem {
                syn::Type::Path( ref path ) => {
                    if match_shallow_path(&path.path).map( |path| path == "str" ).unwrap_or( false ) {
                        ExportType::StrRef
                    } else {
                        match_type(&*ty.elem)
                    }
                },
                syn::Type::Slice( ref slice ) => {
                    ExportType::Slice( (*slice.elem).clone() )
                },
                ref elem => ExportType::WrappedRef( elem.clone() )
            }
        },
        &syn::Type::Path( ref path ) => {
            if let Some(export_type) = match_result_type(&ty) {
                return export_type;
            }

            let name = match match_shallow_path( &path.path ) {
                Some( name ) => name,
                None => return ExportType::Wrapped( ty.clone() )
            };

            match name {
                "i64" => ExportType::Int,
                "u64" => ExportType::Int,
                "i32" => ExportType::Int,
                "u32" => ExportType::Int,
                "i16" => ExportType::Int,
                "u16" => ExportType::Int,
                "i8" => ExportType::Int,
                "u8" => ExportType::Int,

                "f64" => ExportType::Float,
                "f32" => ExportType::Float,

                "String" => ExportType::String,
                "Fn" => ExportType::Fn,
                "PikeFunction" => ExportType::PikeFunction,
                "PikeThing" => ExportType::PikeThing,
                "PikeString" => ExportType::PikeString,
                _ => ExportType::Wrapped( ty.clone() )
            }
        },
        &syn::Type::Tuple( ref tuple ) => {
            if tuple.elems.is_empty() {
                return ExportType::Unit
            }
            ExportType::Unknown(ty.clone())
        },
        &syn::Type::Paren(ref tp) => {
            match_type(&*tp.elem)
        }
        _ => ExportType::Unknown( ty.clone() )
    }
}

fn into_export( ident: syn::Ident, decl: &syn::FnDecl ) -> Export {
    assert!( decl.generics.lifetimes().next().is_none(), "Lifetimes are not yet not supported" );
    assert!( decl.generics.type_params().next().is_none(), "Generics are not supported" );
    assert!( decl.generics.where_clause.is_none(), "`where` clauses are not supported" );
    assert!( decl.variadic.is_none(), "Variadic functions are not supported" );

    let return_ty = match &decl.output {
        &syn::ReturnType::Default => ExportType::Unit,
        &syn::ReturnType::Type( _, ref ty ) => match_type( ty )
    };

    let mut args = Vec::new();
    for (index, arg) in decl.inputs.iter().cloned().enumerate() {
        match arg {
            syn::FnArg::SelfRef(_ty) => {
                let ident = syn::Ident::from("__self");
                args.push (ExportArg {
                    ident,
                    ty: ExportType::SelfRef
                });
            },
            syn::FnArg::SelfValue( .. ) => panic!( "`self` is not supported" ),
            syn::FnArg::Ignored(ty) => {
                let ident = syn::Ident::from( format!( "__arg_{}", index ) );
                args.push( ExportArg {
                    ident,
                    ty: match_type( &ty )
                });
            },
            syn::FnArg::Captured( cap ) => {
                match cap.pat {
                    syn::Pat::Wild( _ ) => {
                        let ident = syn::Ident::from( format!( "__arg_{}", index ) );
                        args.push( ExportArg {
                            ident,
                            ty: match_type( &cap.ty )
                        });
                    },
                    syn::Pat::Ident( pat ) => {
                        assert!( pat.by_ref.is_none(), "`ref` bindings are not supported" );
                        assert!( pat.mutability.is_none(), "`mut` bindings are not supported" );
                        assert!( pat.subpat.is_none(), "Subpatterns are not supported" );

                        args.push( ExportArg {
                            ident: pat.ident,
                            ty: match_type( &cap.ty )
                        });
                    },
                    _ => panic!( "Argument patterns are not supported" )
                }
            },
            syn::FnArg::Inferred( _ ) => panic!( "inferred argument types are not supported" )
        }
    }

    Export {
        ident,
        return_ty,
        args,
        is_constructor: false,
        struct_ty: None
    }
}

fn pike_return_type(return_ty: &ExportType) -> &str {
    match return_ty {
        ExportType::Unit => {
            "void"
        },
        ExportType::Int => {
            "int"
        },
        ExportType::Float => {
            "float"
        },
        ExportType::StrRef => {
            "string"
        },
        ExportType::String => {
            "string"
        },
        ExportType::Fn => {
            "function"
        },
        ExportType::PikeFunction => {
            "function"
        },
        ExportType::PikeThing => {
            "mixed"
        },
        ExportType::PikeString => {
            "string"
        },
        ExportType::Wrapped(_t) => {
            "object"
        },
        ExportType::Result(t) => {
            pike_return_type(&*t)
        },
        _ => { panic!("Unhandled return type"); }
    }
}

fn result_wrapper_code(ty: &ExportType, export: &Export, call: quote::Tokens)
    -> quote::Tokens {

    match ty {
        ExportType::Wrapped(ty) => {
            let ident = ident_from_type(&ty).expect("Got no ident from type");
            let program_var = program_var_name(&ident);
            quote! {
                let prog_ref = #program_var.as_ref();
                match prog_ref.expect("Program var not initialized")
                         .clone_object_with_data(#call) {
                             Ok(val) => {
                                 Ok(val.into())
                             },
                             Err(err) => {
                                 return Err(err)
                             }
                         }
            }
        },

        ExportType::Result(ref inner_ty) => {
            let inner_call = quote! {
                match #call {
                    Ok(val) => {
                        val
                    },
                    Err(err) => {
                        return Err(PikeError::Generic(format!("{:?}", &err)))
                    }
                }
            };

            result_wrapper_code(inner_ty, export, inner_call)
        },

        _ => {
            quote! { Ok(#call.into()) }
        }
    }
}

// Generates code for a normal wrapper function. See also create_wrapper_func.
fn gen_normal_wrapper_func(export_ident: &syn::Ident,
    export: &Export,
    export_args_conversions: Vec<quote::Tokens>,
    fncall: quote::Tokens) -> quote::Tokens {
        let result_conversion =
            result_wrapper_code(&export.return_ty, export, fncall);
        quote! {
                #[doc(hidden)]
                #[no_mangle]
                #[deny(private_no_mangle_fns)]
                #[allow(unused_imports)]
                pub unsafe extern "C" fn #export_ident(args: i32) {
                    let ctx = PikeContext::assume_got_context();
                    let errmsg: Option<String> = {
                        let catch_res = ::std::panic::catch_unwind(|| ->
                            Result<PikeThing, PikeError> {
                                let ctx = PikeContext::assume_got_context();
                                #(#export_args_conversions)*
                                #result_conversion
                            });

                        match catch_res {
                            Ok(ref inner_res) => {
                                match *inner_res {
                                    Ok(ref pt) => {
                                        ctx.push_to_stack(pt.clone(&ctx));
                                        None
                                    }
                                    Err(ref err) => {
                                        Some(format!("{}", &err))
                                    }
                                }
                            }
                            Err(err) => {
                                Some(format!("{:?}", &err))
                            }
                        }
                    };

                    match errmsg {
                        Some(e) => {
                            prepare_error_message(&e);
                            //std::mem::drop(errmsg);
                            ctx.pike_error()
                        }
                        None => {}
                    }
                }
      }
}

// Generates code for a create wrapper function (Pike object constructor).
// The generated code assigns the resulting Rust data to the storage of the
// current object (instead of converting it to a PikeThing and putting it on
// the Pike stack).
fn gen_create_wrapper_func(export_ident: &syn::Ident,
    export: &Export,
    export_args_conversions: Vec<quote::Tokens>,
    fncall: quote::Tokens) -> quote::Tokens {

        let result_conversion = match export.return_ty {
            ExportType::Result(_) => {
                quote! {
                    match #fncall {
                        Ok(val) => {
                            val
                        },
                        Err(err) => {
                            return Err(PikeError::Generic(format!("{:?}", &err)))
                        }
                    }
                }
            }
            _ => {
                fncall
            }
        };
        let struct_ty = export.struct_ty;

        quote! {
                #[doc(hidden)]
                #[no_mangle]
                #[deny(private_no_mangle_fns)]
                #[allow(unused_imports)]
                pub unsafe extern "C" fn #export_ident(args: i32) {
                    let ctx = PikeContext::assume_got_context();
                    let errmsg: Option<String> = {
                        let catch_res = ::std::panic::catch_unwind(|| ->
                            Result<(), PikeError> {
                                let ctx = PikeContext::assume_got_context();
                                #(#export_args_conversions)*
                                let res = #result_conversion;
                                let cur_pike_obj = PikeObject::<#struct_ty>
                                    ::current_object(&ctx);
                                cur_pike_obj.update_data(res);
                                Ok(())
                            });

                        match catch_res {
                            Ok(ref inner_res) => {
                                match *inner_res {
                                    Ok(_) => {
                                        None
                                    }
                                    Err(ref err) => {
                                        Some(format!("{}", &err))
                                    }
                                }
                            }
                            Err(err) => {
                                Some(format!("{:?}", &err))
                            }
                        }
                    };

                    match errmsg {
                        Some(e) => {
                            prepare_error_message(&e);
                            //std::mem::drop(errmsg);
                            ctx.pike_error()
                        }
                        None => {}
                    }
                }
      }
}

fn process( exports: Vec< Export > ) -> quote::Tokens {
    let mut output = Vec::new();

    for export in exports {
        let mut export_args_idents = Vec::new();
        let mut export_args_conversions = Vec::new();

        let mut pike_args_types = Vec::new();
        let mut self_ref = false;

        let mut num_args = 0i32;
        let mut arg_idx_offset = 0i32;

        for (index, arg) in export.args.iter().enumerate() {
            let export_arg_ident = arg.ident.clone();
            let mut tmp_arg_name = export_arg_ident.to_string();
            tmp_arg_name.push_str("_tmp");
            let tmp_arg_ident = syn::Ident::new(&tmp_arg_name,
            export_arg_ident.span());
            let arg_idx = (index as i32) - arg_idx_offset;
            let mut add_arg = true;

            match arg.ty {
                ExportType::SelfRef => {
                    let struct_type = match export.struct_ty {
                        Some(t) => {
                            self_ref = true;
                            t
                        },
                        None => { panic!("Self ref on non-struct method."); }
                    };
                    // FIXME: Check that current_object is an instance of the
                    // expected Pike program.
                    export_args_conversions.push(quote! {
                        let cur_pike_obj = PikeObject::<#struct_type>::current_object(&ctx);
                        let #export_arg_ident: &mut #struct_type = cur_pike_obj.wrapped();
                    });
                    add_arg = false;
                    arg_idx_offset = 1;
                },
                ExportType::Int => {
                    pike_args_types.push("int");
                    export_args_conversions.push(quote! {
                        let #export_arg_ident: PikeInt = match ctx.get_from_stack((-args + #arg_idx) as isize) {
                            PikeThing::Int(res) => { res }
                            _ => { return Err(PikeError::Args("Wrong argument type, expected int.".to_string())); }
                        };
                    });

                },
                ExportType::Float => {
                    pike_args_types.push("float");

                    export_args_conversions.push(quote! {
                        let #export_arg_ident: PikeFloat = match ctx.get_from_stack((-args + #arg_idx) as isize) {
                            PikeThing::Float(res) => { res }
                            _ => { return Err(PikeError::Args("Wrong argument type, expected float.".to_string())); }
                        };
                    });
                },
                ExportType::String => {
                    pike_args_types.push("string");

                    export_args_conversions.push(quote! {
                        let #export_arg_ident: PikeString = match ctx.get_from_stack((-args + #arg_idx) as isize) {
                            PikeThing::PikeString(res) => { res.unwrap(&ctx) }
                            _ => { return Err(PikeError::Args("Wrong argument type, expected string.".to_string())); }
                        };
                    });
                },
                ExportType::StrRef => {
                    pike_args_types.push("string");

                    export_args_conversions.push(quote! {
                        let #tmp_arg_ident: String = match ctx.get_from_stack((-args + #arg_idx) as isize) {
                            PikeThing::PikeString(res) => { res.unwrap(&ctx).into() }
                            _ => { return Err(PikeError::Args("Wrong argument type, expected string.".to_string())); }
                        };
                        let #export_arg_ident: &str = &#tmp_arg_ident;
                    });
                },
                ExportType::Fn => {
                    pike_args_types.push("function");

                    export_args_conversions.push(quote! {
                        let #export_arg_ident = match ctx.get_from_stack((-args + #arg_idx) as isize) {
                            PikeThing::Function(res) => { res.unwrap(&ctx) }
                            _ => { return Err(PikeError::Args("Wrong argument type, expected function.".to_string())); }
                        };
                    });
                },
                ExportType::PikeFunction => {
                    pike_args_types.push("function");
                    export_args_conversions.push(quote! {
                        let #export_arg_ident = match ctx.get_from_stack((-args + #arg_idx) as isize) {
                            PikeThing::Function(res) => { res.unwrap(&ctx) }
                            _ => { return Err(PikeError::Args("Wrong argument type, expected function.".to_string())); }
                        };
                    });
                },
                ExportType::PikeThing => {
                    pike_args_types.push("mixed");
                    export_args_conversions.push(quote! {
                        let #export_arg_ident = ctx.get_from_stack((-args + #arg_idx) as isize);
                    });
                },
                ExportType::PikeString => {
                    pike_args_types.push("string");
                    export_args_conversions.push(quote! {
                        let #export_arg_ident = match ctx.get_from_stack((-args + #arg_idx) as isize) {
                            PikeThing::PikeString(res) => { res.unwrap(&ctx) }
                            _ => { return Err(PikeError::Args("Wrong argument type, expected string.".to_string())); }
                        };
                    });
                },
                ExportType::WrappedRef(ref _t) => {
                    pike_args_types.push("object");
                    export_args_conversions.push(quote! {
                        let #export_arg_ident = match ctx.get_from_stack((-args + #arg_idx) as isize) {
                            PikeThing::PikeObject(res) => { res.unwrap(&ctx) }
                            _ => { return Err(PikeError::Args("Wrong argument type, expected string.".to_string())); }
                        };
                    });
                },
                _ => { panic!("Unhandled argument type"); }
            }
            if add_arg {
                export_args_idents.push( quote! { #export_arg_ident.into() } );
                num_args += 1;
            }
        }

        export_args_conversions.insert(0, quote! {
            if args != #num_args {
                return Err(PikeError::Args("Wrong number of arguments".to_string()));
            }
        });


        if pike_args_types.len() == 0 {
            pike_args_types.push("void");
        }

        let original_ident = export.ident.clone();
        let fncall = match export.struct_ty {
            Some(ty) => {
                if self_ref {
                    quote! { __self.#original_ident(#(#export_args_idents),*) }
                } else {
                    quote! { #ty::#original_ident(#(#export_args_idents),*) }
                }
            },
            None => { quote! { #original_ident(#(#export_args_idents),*) } }
        };

        let pike_ident = format!("{}", export.ident);
        let export_ident = syn::Ident::from(format!("rustfn_{}", &original_ident));

        if export.is_constructor {
            output.push(gen_create_wrapper_func(&export_ident, &export,
                export_args_conversions, fncall));
        } else {
            output.push(gen_normal_wrapper_func(&export_ident, &export,
                export_args_conversions, fncall));
        }

        let pike_func_type: String = format!("function({}:{})",
            pike_args_types.join(","),
            pike_return_type(&export.return_ty));
        EXPORT_FUNC_INITS.with(|e| {
            let ref mut a = *e.borrow_mut();
            a.push(
                quote! {
                    PikeProgram::<()>::add_pike_func(#pike_ident,
                        #pike_func_type,
                        #export_ident);
                })
        });
    }

    quote! { #(#output)* }
}

fn handle_item_impl(mut orig_item: syn::Item) -> TokenStream {
    let mut exports = Vec::new();
    let struct_ty;

    {
        let item = match orig_item {
            syn::Item::Impl(ref mut i) => { i },
            _ => { panic!("Wrong type") }
        };

        struct_ty = match *item.self_ty {
            syn::Type::Path(ref typath) => {
                Some(typath.path.segments.last().as_ref().unwrap().value().ident.clone())
            },
            _ => { panic!("impl block must have a name"); }
        };

        EXPORT_FUNC_INITS.with(|e| {
            let ref mut a = *e.borrow_mut();
            a.push(
                quote! {
                    let ctx;
                    unsafe {
                        ctx = PikeContext::assume_got_context();
                        PikeProgram::<#struct_ty>::start_new_program(file!(), line!());
                    }
                }
            );
        });

        let impls = &mut item.items;
        for iimpl in impls.iter_mut() {
            if let syn::ImplItem::Method(ref mut meth) = iimpl {
                let name = meth.sig.ident.clone();
                let mut export = into_export(name, &meth.sig.decl);
                export.struct_ty = struct_ty;
                if export.ident.as_ref() == "create" {
                    export.is_constructor = true;
                }
                exports.push(export);
            }
        }
    }

    let generated = process(exports);
    let program_var = program_var_name(&struct_ty.unwrap());

    GLOBAL_DEFS.with(|e| {
        let ref mut a = *e.borrow_mut();
        a.push(
            quote! {
                static mut #program_var: Option<PikeProgramRef<#struct_ty>> = None;
            }
        )
    });

    EXPORT_FUNC_INITS.with(|e| {
        let ref mut a = *e.borrow_mut();
        let class_ident = format!("{}", struct_ty.unwrap());
        a.push(
            quote! {
                let new_class_prog = PikeProgram::<#struct_ty>::finish_program(&ctx);
                PikeProgram::add_program_constant(#class_ident, new_class_prog.clone());
                unsafe {
                    // We know that Pike's compiler lock protects us from
                    // data races, so we're doing an unsafe mutable assignment
                    // here.
                    #program_var = Some((&new_class_prog).into());
                }
            }
        );
    });

    let output = quote! {
        #orig_item
        #generated
    };

    output.into()
}

#[proc_macro_attribute]
pub fn pike_export(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let mut exports = Vec::new();
    let item: syn::Item = syn::parse(input).unwrap();

    if !attrs.is_empty() {
        panic!( "Extra attributes are not supported in `#[pike_export]`!" );
    }

    match item {
        syn::Item::Fn(ref function) => {
            exports.push(into_export(function.ident.clone(), &function.decl));
        },
        syn::Item::Impl(_) => {
            return handle_item_impl (item);
        }
        _ => panic!( "`#[pike_export]` attached to an unsupported element!" )
    }

    let generated = process(exports);
    let output = quote! {
        #item
        #generated
    };

    output.into()
}

fn program_var_name(struct_name: &syn::Ident) -> syn::Ident {
    format!("{}_PROGRAM", struct_name).to_uppercase().into()
}

thread_local! {
    static GLOBAL_DEFS: ::std::cell::RefCell<Vec<quote::Tokens>> =
      ::std::cell::RefCell::new(vec![]);
}

thread_local! {
    static EXPORT_FUNC_INITS: ::std::cell::RefCell<Vec<quote::Tokens>> =
      ::std::cell::RefCell::new(vec![]);
}

#[proc_macro]
pub fn pike_func_inits(_input: TokenStream) -> TokenStream {
    let mut inits: Vec<quote::Tokens> = vec![];
    EXPORT_FUNC_INITS.with(|e| {
        let ref mut v = &*e.borrow_mut();
        inits = v.clone().to_vec();
        //v.clear();
    });
    let output = quote! {
        #(#inits)*
    };
    output.into()
}

#[proc_macro]
pub fn init_pike_module(_input: TokenStream) -> TokenStream {
    let mut global_defs: Vec<quote::Tokens> = vec![];
    GLOBAL_DEFS.with(|e| {
        let ref mut v = &*e.borrow_mut();
        global_defs = v.clone().to_vec();
        //v.clear();
    });

    let mut inits: Vec<quote::Tokens> = vec![];
    EXPORT_FUNC_INITS.with(|e| {
        let ref mut v = &*e.borrow_mut();
        inits = v.clone().to_vec();
        //v.clear();
    });

    let output = quote! {
        #(#global_defs)*

        #[no_mangle]
        pub extern "C" fn pike_module_init() {
            #(#inits)*
        }
        #[no_mangle]
        pub extern "C" fn pike_module_exit() {

        }
    };
    output.into()
}
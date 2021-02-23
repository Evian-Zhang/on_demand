//! This crate provides a macro to generate aux macros for on-demand usage.
//!
//! # Examples
//!
//! ```rust
//! use on_demand::generate_on_demand_macro;
//!
//! fn foo() {
//!     generate_on_demand_macro!(a: usize = None, {
//!         println!("a");
//!         1
//!     });
//!     generate_on_demand_macro!(b: usize = None, {
//!         println!("b");
//!         let a_data = on_demand_get_a!();
//!         2 + *a_data
//!     });
//!     generate_on_demand_macro!(c: usize = None, {
//!         println!("c");
//!         let a_data = on_demand_get_a!();
//!         let b_data = on_demand_get_b!();
//!         3 + *a_data + *b_data
//!     });
//!
//!     let c_data = on_demand_get_c!();
//!     assert_eq!(*c_data, 6);
//! }
//! ```
//!
//! After calling `generate_on_demand_macro` to the variable (for example, `a`),
//! three new macros `on_demand_get_a`, `on_demand_get_a_mut` and `on_demand_into_a` are generated.
//! When `on_demand_get_a` is called, it determines whether `a` has been calculated. If it is,
//! then returns the reference to its data, otherwise calls the expression given as the second
//! parameter of `generate_on_demand_macro`, then assigns the returned value to `a`. The other two
//! generated macros do the similar job, but returns the mutable reference to, or takes ownership
//! of `a`.
//!
//! In all, this means calculating `a` lazily using the expression.
//!
//! # Notes
//!
//! The returned value of `on_demand_get_a` is essentially a [`Ref`][std::cell::Ref], and
//! `on_demand_get_a_mut` is [`RefMut`][std::cell::RefMut]. So remember to dereference it
//! when necessary.
//!
//! When calling this macro, we assign a default value by `generate_on_demand_macro!(a: MyType
//! = default_value_of_optional_my_type), { /* block */ });`. The `default_value_of_optional_my_type` should
//! be of type `Option<MyType>`. If it is not `None`, then `a` is considered calculated already, and the expression
//! given in block will never be called.
//!
//! There are some crates providing lazy feature, such as [spin](https://crates.io/crates/spin).
//! However, those methods are based on closure, which could not handle some situations such as below
//! (error handlings are omitted for simplicity):
//!
//! ```rust
//! # use on_demand::generate_on_demand_macro;
//! # use std::io::{Seek, Read, SeekFrom};
//! fn foo<T: Seek + Read>(binary: &mut T) {
//!     generate_on_demand_macro!(a: u32 = None, {
//!         let mut buf = [0; 4];
//!         binary.seek(SeekFrom::Start(0)).unwrap();
//!         binary.read(&mut buf).unwrap();
//!         u32::from_be_bytes(buf)
//!     });
//!     generate_on_demand_macro!(b: u32 = None, {
//!         let a_data = on_demand_get_a!();
//!         let mut buf = [0; 4];
//!         binary.seek(SeekFrom::Start(0)).unwrap();
//!         binary.read(&mut buf).unwrap();
//!         u32::from_be_bytes(buf) + *a_data
//!     });
//!     generate_on_demand_macro!(c: u32 = None, {
//!         let a_data = on_demand_get_a!();
//!         let b_data = on_demand_get_b!();
//!         let mut buf = [0; 4];
//!         binary.seek(SeekFrom::Start(0)).unwrap();
//!         binary.read(&mut buf).unwrap();
//!         u32::from_be_bytes(buf) + *a_data + *b_data
//!     });
//!     let a_data = on_demand_get_a!();
//!     let b_data = on_demand_get_b_mut!();
//!     drop(b_data); // drop here since c will take ownership
//!     let c_data = on_demand_into_c!();
//! }
//! ```
//!
//! `binary` is considered as uniquely borrowed if closure is used, then the borrow checker
//! won't allow us do above things. However, macros can do such things.

#[doc(hidden)]
pub use paste;

/// Macro to generate on-demand macro
#[macro_export]
macro_rules! generate_on_demand_macro {
    ($var: ident: $Inner: ty = $default_value: expr, $getter: expr) => {
        let $var: ::std::cell::RefCell<::std::option::Option<$Inner>> =
            ::std::cell::RefCell::new($default_value);
        $crate::paste::paste! {
            #[allow(unused_macros)]
            macro_rules! [<on_demand_get_ $var>] {
                () => {{
                    let is_some = $var.borrow().is_some();
                    if !is_some {
                        *($var.borrow_mut()) = Some({$getter});
                    }
                    ::std::cell::Ref::map($var.borrow(), |var| {
                        if let Some(data) = var.as_ref() {
                            data
                        } else {
                            unreachable!()
                        }
                    })
                }};
            }
            #[allow(unused_macros)]
            macro_rules! [<on_demand_get_ $var _mut>] {
                () => {{
                    let is_some = $var.borrow().is_some();
                    if !is_some {
                        *($var.borrow_mut()) = Some({$getter});
                    }
                    ::std::cell::RefMut::map($var.borrow_mut(), |var| {
                        if let Some(data) = var.as_mut() {
                            data
                        } else {
                            unreachable!()
                        }
                    })
                }};
            }
            #[allow(unused_macros)]
            macro_rules! [<on_demand_into_ $var>] {
                () => {{
                    if let Some(data) = $var.into_inner() {
                        data
                    } else {
                        {$getter}
                    }
                }};
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::generate_on_demand_macro;
    use std::io::{Cursor, Read, Seek, SeekFrom};

    #[test]
    fn test_reader() {
        let mut binary = Cursor::new([0x0u8, 0x1u8, 0x2u8, 0x3u8]);
        generate_on_demand_macro!(a: u32 = None, {
            let mut buf = [0; 4];
            binary.seek(SeekFrom::Start(0)).unwrap();
            binary.read(&mut buf).unwrap();
            u32::from_le_bytes(buf)
        });
        generate_on_demand_macro!(b: u32 = None, {
            let a_data = on_demand_get_a!();
            let mut buf = [0; 4];
            binary.seek(SeekFrom::Start(0)).unwrap();
            binary.read(&mut buf).unwrap();
            u32::from_le_bytes(buf) + *a_data
        });
        generate_on_demand_macro!(c: u32 = None, {
            let a_data = on_demand_get_a!();
            let b_data = on_demand_get_b!();
            let mut buf = [0; 4];
            binary.seek(SeekFrom::Start(0)).unwrap();
            binary.read(&mut buf).unwrap();
            u32::from_le_bytes(buf) + *a_data + *b_data
        });
        let a_data = on_demand_get_a!();
        assert_eq!(*a_data, 0x3020100);
        let b_data = on_demand_get_b_mut!();
        assert_eq!(*b_data, 0x6040200);
        drop(b_data); // drop here since c will take ownership
        let c_data = on_demand_into_c!();
        assert_eq!(c_data, 0xc080400);
    }
}

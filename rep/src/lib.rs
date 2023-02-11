//! Rep is a small tool for checking representation/class invariants

use std::ops::Deref;

pub use log::Level::Error;
pub use log::{error, log_enabled};
pub use rep_derive::check_rep;
pub use rep_derive::*;

/// A trait for checking invariants on independent fields of a `struct`
pub trait CheckIndieFields {
    /// Append any errors to `e`
    fn check_indie_fields(&self, _e: &mut RepErrors) {}
}

pub trait CheckRep: CheckIndieFields + CheckFields {
    /// Asserts that self is correct
    fn check_rep(&self) {
        let mut errors = RepErrors::new();
        self.check_indie_fields(&mut errors);
        self.check_fields(&mut errors);
        if errors.is_empty() {
            return;
        }
        if log_enabled!(Error) {
            for error in errors.iter() {
                error!("representation invariant violated: {:?}", error);
            }
        } else {
            panic!("representation invariant violated: {:?}", errors);
        }
    }
}

/// A trait for checking invariants on interrelated fields of a `struct`
pub trait CheckFields {
    /// Append any errors to `e`
    fn check_fields(&self, _e: &mut RepErrors) {}
}

#[derive(Debug)]
pub struct RepErrors {
    errors: Vec<String>,
}

impl RepErrors {
    pub fn new() -> RepErrors {
        RepErrors { errors: vec![] }
    }

    pub fn add(&mut self, error: String) {
        self.errors.push(error);
    }
}

impl Deref for RepErrors {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.errors
    }
}

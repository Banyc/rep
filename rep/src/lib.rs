//! Rep is a small tool for checking representation/class invariants

use std::ops::Deref;

pub use log::Level::Error;
pub use log::{error, log_enabled};
pub use rep_derive::check_rep;
pub use rep_derive::*;

/// A trait for representation checking
pub trait CheckRep {
    /// Returns Ok if representation is correct, vector of errors otherwise
    fn correctness(&self, e: &mut RepErrors);

    /// Asserts that self is correct
    fn check_rep(&self) {
        let mut errors = RepErrors::new();
        self.correctness(&mut errors);
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

/// A trait for adding extra rep-checking functionality to a data structure with `CheckRep` implemented
pub trait CustomCheckRep {
    /// Returns Ok if representation is correct, vector of errors otherwise
    fn collect_errors(&self, _e: &mut RepErrors) {}
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

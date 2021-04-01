#![allow(clippy::new_without_default)]
#![allow(clippy::upper_case_acronyms)]

#[macro_use]
extern crate num_derive;

pub mod header;
pub mod packet;
pub mod packet_buffer;
pub mod query;
pub mod query_type;
pub mod resolve;
pub mod resource_record;
pub mod return_code;

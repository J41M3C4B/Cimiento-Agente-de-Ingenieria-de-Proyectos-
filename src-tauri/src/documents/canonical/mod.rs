//! Reading a call into the canonical schema (ADR-015).
//!
//! The rail (ADR-013/014) let the code decide what was relevant from the format of the documents; it
//! solved the calls it was tuned on and failed on the next. This is the opposite split:
//!
//! * the **contract** (`contract`) says what is needed from every call, whatever its funder or layout;
//! * the **package** (`package`) turns the files into numbered pages and a page map, with no rule about
//!   how a call is written;
//! * the **retriever** (`retrieve`) picks, for each block of the schema, the pages worth reading (by
//!   meaning: embeddings, or a plain lexical score), so the model never receives 120 pages for a block;
//! * the **model** fills the block with quotes copied from the pages;
//! * the **code** (`assemble`) verifies every quote against its page, reads dates, amounts and
//!   percentages from the quotes, validates against the schema and keeps contradictions instead of
//!   choosing;
//! * `run` wires them in the two shapes to compare: the whole schema in one call, or one call per block.
//!
//! Nothing here may depend on how one funder writes its calls: the code is measured on all of them and
//! is never adjusted to one.

pub mod assemble;
pub mod card;
pub mod contract;
pub mod normalize;
pub mod package;
pub mod retrieve;
pub mod run;
pub mod summary;

#[cfg(test)]
mod live_tests;

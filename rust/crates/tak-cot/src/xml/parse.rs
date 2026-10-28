// SPDX-License-Identifier: Apache-2.0

//! The bounded, single-pass parser that builds a `Document` from a `&str`
//! and enforces every `Limits` value before the allocation it bounds (WU3).

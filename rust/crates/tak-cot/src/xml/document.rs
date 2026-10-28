// SPDX-License-Identifier: Apache-2.0

//! The arena document model: nodes in one `Vec` linked by `u32` indices, so
//! building, walking, cloning and dropping never recurse (WU3).

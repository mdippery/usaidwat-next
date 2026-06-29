// SPDX-License-Identifier: Apache-2.0
// Copyright (C) 2025-2026 Michael Dippery <michael@monkey-robot.com>

#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]

pub mod count;
pub mod filter;
pub mod reddit;
pub mod summary;
pub mod text;
pub mod view;

#[doc(hidden)]
pub mod cli;

#[cfg(test)]
mod test_utils;

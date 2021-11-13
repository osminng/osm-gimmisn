/*
 * Copyright 2021 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Provides the 'parse_access_log' cmdline tool.

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let ctx = rust::context::Context::new("")?;
    rust::parse_access_log::main(&args, &mut std::io::stdout(), &ctx)?;

    Ok(())
}

/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the cache module.

use super::*;
use context::FileSystem;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// Tests get_missing_housenumbers_json(): the cached case.
///
/// The non-cached case is covered by higher level
/// wsgi_json::tests::test_missing_housenumbers_view_result_json().
#[test]
fn test_get_missing_housenumbers_json() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let json_cache_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/cache-gazdagret.json", &json_cache_value),
        ],
    );
    file_system.set_files(&files);
    file_system
        .write_from_string(
            "{'cached':'yes'}",
            &ctx.get_abspath("workdir/cache-gazdagret.json"),
        )
        .unwrap();
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/cache-gazdagret.json"),
        Rc::new(RefCell::new(
            time::OffsetDateTime::from_unix_timestamp(9999999999).unwrap(),
        )),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation("gazdagret").unwrap();

    let ret = get_missing_housenumbers_json(&mut relation).unwrap();

    assert_eq!(ret, "{'cached':'yes'}");
}

/// Tests get_additional_housenumbers_json(): the cached case.
///
/// The non-cached case is covered by higher level
/// wsgi_json::tests::test_additional_housenumbers_view_result_json().
#[test]
fn test_get_additional_housenumbers_json() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let json_cache_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/additional-cache-gazdagret.json", &json_cache_value),
        ],
    );
    file_system.set_files(&files);
    file_system
        .write_from_string(
            "{'cached':'yes'}",
            &ctx.get_abspath("workdir/additional-cache-gazdagret.json"),
        )
        .unwrap();
    let mut mtimes: HashMap<String, Rc<RefCell<time::OffsetDateTime>>> = HashMap::new();
    mtimes.insert(
        ctx.get_abspath("workdir/additional-cache-gazdagret.json"),
        Rc::new(RefCell::new(
            time::OffsetDateTime::from_unix_timestamp(9999999999).unwrap(),
        )),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation("gazdagret").unwrap();

    let ret = get_additional_housenumbers_json(&mut relation).unwrap();

    assert_eq!(ret, "{'cached':'yes'}");
}

/// Tests is_cache_current()
#[test]
fn test_is_cache_current() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let cache_path = "workdir/gazdagret.json.cache";
    file_system.set_hide_paths(&[cache_path.to_string()]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    let ret = is_cache_current(&ctx, &cache_path, &[]).unwrap();

    assert_eq!(ret, false);
}

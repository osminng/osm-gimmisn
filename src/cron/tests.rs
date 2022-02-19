/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the cron module.

use super::*;
use context::FileSystem;
use std::cell::RefCell;
use std::io::Seek;
use std::io::SeekFrom;
use std::rc::Rc;
use std::sync::Arc;

/// Tests overpass_sleep(): the case when no sleep is needed.
#[test]
fn test_overpass_sleep_no_sleep() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/status",
        /*data_path=*/ "",
        /*result_path=*/ "tests/network/overpass-status-happy.txt",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);

    overpass_sleep(&ctx);

    let time = time_arc
        .as_any()
        .downcast_ref::<context::tests::TestTime>()
        .unwrap();
    assert_eq!(time.get_sleep(), 0);
}

/// Tests overpass_sleep(): the case when sleep is needed.
#[test]
fn test_overpass_sleep_need_sleep() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-status-wait.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-status-happy.txt",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);

    overpass_sleep(&ctx);

    let time = time_arc
        .as_any()
        .downcast_ref::<context::tests::TestTime>()
        .unwrap();
    assert_eq!(time.get_sleep(), 12);
}

/// Tests update_ref_housenumbers().
#[test]
fn test_update_ref_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "refsettlement": "42",
                "refcounty": "01",
                "refsettlement": "011",
            },
            "ujbuda": {
                "refsettlement": "42",
            },
        },
        "relation-gazdagret.yaml": {
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            },
        },
        "relation-ujbuda.yaml": {
            "missing-streets": "only",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_file1 = context::tests::TestFileSystem::make_file();
    let ref_file2 = context::tests::TestFileSystem::make_file();
    let ref_housenumbers_cache = context::tests::TestFileSystem::make_file();
    let ref_housenumbers2_cache = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            (
                "refdir/hazszamok_20190511.tsv-01-v1.cache",
                &ref_housenumbers_cache,
            ),
            (
                "refdir/hazszamok_kieg_20190808.tsv-01-v1.cache",
                &ref_housenumbers2_cache,
            ),
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_file1,
            ),
            (
                "workdir/street-housenumbers-reference-ujbuda.lst",
                &ref_file2,
            ),
        ],
    );
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    let path = ctx.get_abspath("workdir/street-housenumbers-reference-gazdagret.lst");
    mtimes.insert(path.to_string(), Rc::new(RefCell::new(0_f64)));
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();

    update_ref_housenumbers(&ctx, &mut relations, /*update=*/ true).unwrap();

    let mtime = ctx.get_file_system().getmtime(&path).unwrap();
    assert!(mtime > 0_f64);

    update_ref_housenumbers(&ctx, &mut relations, /*update=*/ false).unwrap();

    assert_eq!(ctx.get_file_system().getmtime(&path).unwrap(), mtime);
    let actual = context::tests::TestFileSystem::get_content(&ref_file1);
    let expected = std::fs::read_to_string(&path).unwrap();
    assert_eq!(actual, expected);
    // Make sure housenumber ref is not created for the streets=only case.
    let mut guard = ref_file2.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap(), 0);
}

/// Tests update_ref_streets().
#[test]
fn test_update_ref_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "refsettlement": "42",
                "refcounty": "01",
                "refsettlement": "011",
            },
            "gellerthegy": {
                "refsettlement": "42",
            },
        },
        "relation-gazdagret.yaml": {
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            },
        },
        "relation-gellerthegy.yaml": {
            "missing-streets": "no",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let streets_ref_myrelation1 = context::tests::TestFileSystem::make_file();
    let streets_ref_myrelation2 = context::tests::TestFileSystem::make_file();
    let ref_streets_cache = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "workdir/streets-reference-gazdagret.lst",
                &streets_ref_myrelation1,
            ),
            (
                "workdir/streets-reference-gellerthegy.lst",
                &streets_ref_myrelation2,
            ),
            ("refdir/utcak_20190514.tsv.cache", &ref_streets_cache),
        ],
    );
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    let path = ctx.get_abspath("workdir/streets-reference-gazdagret.lst");
    mtimes.insert(path.to_string(), Rc::new(RefCell::new(0_f64)));
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();

    update_ref_streets(&ctx, &mut relations, /*update=*/ true).unwrap();

    let mtime = ctx.get_file_system().getmtime(&path).unwrap();
    assert!(mtime > 0_f64);

    update_ref_streets(&ctx, &mut relations, /*update=*/ false).unwrap();

    assert_eq!(ctx.get_file_system().getmtime(&path).unwrap(), mtime);
    let actual = context::tests::TestFileSystem::get_content(&streets_ref_myrelation1);
    let expected = std::fs::read_to_string(&path).unwrap();
    assert_eq!(actual, expected);
    // Make sure street ref is not created for the streets=no case.
    let mut guard = streets_ref_myrelation2.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap(), 0);
}

/// Tests update_missing_housenumbers().
#[test]
fn test_update_missing_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
                "refcounty": "01",
                "refsettlement": "011",
            },
            "ujbuda": {
                "osmrelation": 2702687,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-gazdagret.yaml": {
        },
        "relation-ujbuda.yaml": {
            "missing-streets": "only",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let count_file1 = context::tests::TestFileSystem::make_file();
    let count_file2 = context::tests::TestFileSystem::make_file();
    let html_cache1 = context::tests::TestFileSystem::make_file();
    let html_cache2 = context::tests::TestFileSystem::make_file();
    let txt_cache = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret.percent", &count_file1),
            ("workdir/ujbuda.percent", &count_file2),
            ("workdir/gazdagret.htmlcache.en", &html_cache1),
            ("workdir/gazdagret.htmlcache.hu", &html_cache2),
            ("workdir/gazdagret.txtcache", &txt_cache),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let path1 = ctx.get_abspath("workdir/gazdagret.percent");
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(path1.to_string(), Rc::new(RefCell::new(0_f64)));
    mtimes.insert(
        ctx.get_abspath("workdir/gazdagret.htmlcache.en"),
        Rc::new(RefCell::new(0_f64)),
    );
    mtimes.insert(
        ctx.get_abspath("workdir/gazdagret.htmlcache.hu"),
        Rc::new(RefCell::new(0_f64)),
    );
    mtimes.insert(
        ctx.get_abspath("workdir/gazdagret.txtcache"),
        Rc::new(RefCell::new(0_f64)),
    );
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let expected: String = "36.36".into();

    update_missing_housenumbers(&ctx, &mut relations, /*update=*/ true).unwrap();

    let expected_mtime = file_system_arc.getmtime(&path1).unwrap();
    assert_eq!(expected_mtime > 0_f64, true);

    update_missing_housenumbers(&ctx, &mut relations, /*update=*/ false).unwrap();

    let actual_mtime = file_system_arc.getmtime(&path1).unwrap();
    assert_eq!(actual_mtime, expected_mtime);
    let actual = context::tests::TestFileSystem::get_content(&count_file1);
    assert_eq!(actual, expected);
    // Make sure housenumber stat is not created for the streets=only case.
    let mut guard = count_file2.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, false);
}

/// Tests update_missing_streets().
#[test]
fn test_update_missing_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
                "refcounty": "01",
                "refsettlement": "011",
            },
            "gellerthegy": {
                "osmrelation": 2702687,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-gazdagret.yaml": {
        },
        "relation-gellerthegy.yaml": {
            "missing-streets": "no",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let count_file1 = context::tests::TestFileSystem::make_file();
    let count_file2 = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret-streets.percent", &count_file1),
            ("workdir/gellerthegy-streets.percent", &count_file2),
        ],
    );
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    let path1 = ctx.get_abspath("workdir/gazdagret-streets.percent");
    mtimes.insert(path1.to_string(), Rc::new(RefCell::new(0_f64)));
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let expected: String = "50.00".into();

    update_missing_streets(&mut relations, /*update=*/ true).unwrap();

    let expected_mtime = file_system_arc.getmtime(&path1).unwrap();
    assert_eq!(expected_mtime > 0_f64, true);

    update_missing_streets(&mut relations, /*update=*/ false).unwrap();

    let actual_mtime = file_system_arc.getmtime(&path1).unwrap();
    assert_eq!(actual_mtime, expected_mtime);
    let actual = context::tests::TestFileSystem::get_content(&count_file1);
    assert_eq!(actual, expected);
    // Make sure street stat is not created for the streets=no case.
    let mut guard = count_file2.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, false);
}

/// Tests update_additional_streets().
#[test]
fn test_update_additional_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
                "refcounty": "01",
                "refsettlement": "011",
            },
            "gellerthegy": {
                "osmrelation": 2702687,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-gazdagret.yaml": {
            "osm-street-filters": ["Second Only In OSM utca"],
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            },
        },
        "relation-gellerthegy.yaml": {
            "missing-streets": "no",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let count_file1 = context::tests::TestFileSystem::make_file();
    let count_file2 = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/gazdagret-additional-streets.count", &count_file1),
            ("workdir/gellerthegy-additional-streets.count", &count_file2),
        ],
    );
    let path1 = ctx.get_abspath("workdir/gazdagret-additional-streets.count");
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    mtimes.insert(path1.to_string(), Rc::new(RefCell::new(0_f64)));
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let expected: String = "1".into();
    update_additional_streets(&mut relations, /*update=*/ true).unwrap();
    let mtime = file_system_arc.getmtime(&path1).unwrap();

    update_additional_streets(&mut relations, /*update=*/ false).unwrap();

    assert_eq!(file_system_arc.getmtime(&path1).unwrap(), mtime);
    let actual = context::tests::TestFileSystem::get_content(&count_file1);
    assert_eq!(actual, expected);
    // Make sure street stat is not created for the streets=no case.
    let mut guard = count_file2.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, false);
}

/// Tests update_osm_housenumbers().
#[test]
fn test_update_osm_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-housenumbers-gazdagret.csv",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let path = ctx.get_abspath("workdir/street-housenumbers-gazdagret.csv");
    let expected = std::fs::read_to_string(&path).unwrap();

    update_osm_housenumbers(&ctx, &mut relations, /*update=*/ true).unwrap();

    let mtime = ctx.get_file_system().getmtime(&path).unwrap();

    update_osm_housenumbers(&ctx, &mut relations, /*update=*/ false).unwrap();

    assert_eq!(ctx.get_file_system().getmtime(&path).unwrap(), mtime);
    let actual = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(actual, expected);
}

/// Tests update_osm_housenumbers(): the case when we keep getting HTTP errors.
#[test]
fn test_update_osm_housenumbers_http_error() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let path = ctx.get_abspath("workdir/street-housenumbers-gazdagret.csv");
    let expected = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    update_osm_housenumbers(&ctx, &mut relations, /*update=*/ true).unwrap();
    // Make sure that in case we keep getting errors we give up at some stage and
    // leave the last state unchanged.
    let actual = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(actual, expected);
}

/// Tests update_osm_housenumbers(): the case when we ask for CSV but get XML.
#[test]
fn test_update_osm_housenumbers_xml_as_csv() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass.xml",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let path = ctx.get_abspath("workdir/street-housenumbers-gazdagret.csv");
    let expected = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    update_osm_housenumbers(&ctx, &mut relations, /*update=*/ true).unwrap();
    let actual = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(actual, expected);
}

/// Tests update_osm_streets().
#[test]
fn test_update_osm_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-streets-gazdagret.csv",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let osm_streets_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/streets-gazdagret.csv", &osm_streets_value),
        ],
    );
    let mut mtimes: HashMap<String, Rc<RefCell<f64>>> = HashMap::new();
    let path = ctx.get_abspath("workdir/streets-gazdagret.csv");
    mtimes.insert(path.to_string(), Rc::new(RefCell::new(0_f64)));
    let mut file_system = context::tests::TestFileSystem::new();
    file_system.set_files(&files);
    file_system.set_mtimes(&mtimes);
    let file_system_arc: Arc<dyn FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();

    update_osm_streets(&ctx, &mut relations, /*update=*/ true).unwrap();

    let mtime = ctx.get_file_system().getmtime(&path).unwrap();
    assert!(mtime > 0_f64);

    update_osm_streets(&ctx, &mut relations, /*update=*/ false).unwrap();

    assert_eq!(ctx.get_file_system().getmtime(&path).unwrap(), mtime);

    let actual = context::tests::TestFileSystem::get_content(&osm_streets_value);
    let expected = std::fs::read_to_string(&path).unwrap();
    assert_eq!(actual, expected);
}

/// Tests update_osm_streets(): the case when we keep getting HTTP errors.
#[test]
fn test_update_osm_streets_http_error() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let path = ctx.get_abspath("workdir/streets-gazdagret.csv");
    let expected = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();

    update_osm_streets(&ctx, &mut relations, /*update=*/ true).unwrap();

    // Make sure that in case we keep getting errors we give up at some stage and
    // leave the last state unchanged.
    let actual = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(actual, expected);
}

/// Tests update_osm_streets(): the case when we ask for CSV but get XML.
#[test]
fn test_update_osm_streets_xml_as_csv() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass.xml",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    let path = ctx.get_abspath("workdir/streets-gazdagret.csv");
    let expected = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();

    update_osm_streets(&ctx, &mut relations, /*update=*/ true).unwrap();

    let actual = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(actual, expected);
}

/// Creates a 8 days old file.
fn create_old_file(path: &str) {
    let now = chrono::Local::now();
    let current_time = now.naive_local().timestamp();
    let old_time = current_time - (8 * 24 * 3600);
    let old_access_time = old_time;
    let old_modification_time = old_time;
    std::fs::File::create(path).unwrap();
    utime::set_file_times(path, old_access_time, old_modification_time).unwrap();
}

/// Tests update_stats().
#[test]
fn test_update_stats() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-stats.csv",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);

    let citycount_value = context::tests::TestFileSystem::make_file();
    let count_value = context::tests::TestFileSystem::make_file();
    let topusers_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/2020-05-10.citycount", &citycount_value),
            ("workdir/stats/2020-05-10.count", &count_value),
            ("workdir/stats/2020-05-10.topusers", &topusers_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    // Create a CSV that is definitely old enough to be removed.
    let old_path = ctx.get_abspath("workdir/stats/old.csv");
    create_old_file(&old_path);

    let now = chrono::NaiveDateTime::from_timestamp(ctx.get_time().now(), 0);
    let today = now.format("%Y-%m-%d").to_string();
    let path = ctx.get_abspath(&format!("workdir/stats/{}.csv", today));

    update_stats(&ctx, /*overpass=*/ true).unwrap();

    let actual = String::from_utf8(std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        actual,
        String::from_utf8(std::fs::read("tests/network/overpass-stats.csv").unwrap()).unwrap()
    );

    // Make sure that the old CSV is removed.
    assert_eq!(ctx.get_file_system().path_exists(&old_path), false);

    let num_ref: i64 = std::fs::read_to_string(&ctx.get_abspath("workdir/stats/ref.count"))
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    assert_eq!(num_ref, 300);
}

/// Tests update_stats(): the case when we keep getting HTTP errors.
#[test]
fn test_update_stats_http_error() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let routes = vec![context::tests::URLRoute::new(
        /*url=*/ "https://overpass-api.de/api/status",
        /*data_path=*/ "",
        /*result_path=*/ "tests/network/overpass-status-happy.txt",
    )];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);

    let citycount_value = context::tests::TestFileSystem::make_file();
    let count_value = context::tests::TestFileSystem::make_file();
    let topusers_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/2020-05-10.citycount", &citycount_value),
            ("workdir/stats/2020-05-10.count", &count_value),
            ("workdir/stats/2020-05-10.topusers", &topusers_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let stats_path = ctx.get_abspath("workdir/stats/stats.json");
    if std::path::Path::new(&stats_path).exists() {
        std::fs::remove_file(&stats_path).unwrap();
    }

    update_stats(&ctx, /*overpass=*/ true).unwrap();

    assert_eq!(std::path::Path::new(&stats_path).exists(), true);
}

/// Tests update_stats(): the case when we don't call overpass.
#[test]
fn test_update_stats_no_overpass() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let time = context::tests::make_test_time();
    let time_arc: Arc<dyn context::Time> = Arc::new(time);
    ctx.set_time(&time_arc);
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-status-wait.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-status-happy.txt",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);

    let citycount_value = context::tests::TestFileSystem::make_file();
    let count_value = context::tests::TestFileSystem::make_file();
    let topusers_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/2020-05-10.citycount", &citycount_value),
            ("workdir/stats/2020-05-10.count", &count_value),
            ("workdir/stats/2020-05-10.topusers", &topusers_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);

    update_stats(&ctx, /*overpass=*/ false).unwrap();

    let time = time_arc
        .as_any()
        .downcast_ref::<context::tests::TestTime>()
        .unwrap();
    assert_eq!(time.get_sleep(), 0);
}

/// Tests our_main().
#[test]
fn test_our_main() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-streets-gazdagret.csv",
        ),
        // For update_osm_housenumbers().
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-housenumbers-gazdagret.csv",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let osm_streets_value = context::tests::TestFileSystem::make_file();
    let osm_housenumbers_value = context::tests::TestFileSystem::make_file();
    let ref_streets_value = context::tests::TestFileSystem::make_file();
    let ref_housenumbers_value = context::tests::TestFileSystem::make_file();
    let missing_streets_value = context::tests::TestFileSystem::make_file();
    let missing_housenumbers_value = context::tests::TestFileSystem::make_file();
    let additional_streets_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("workdir/streets-gazdagret.csv", &osm_streets_value),
            (
                "workdir/street-housenumbers-gazdagret.csv",
                &osm_housenumbers_value,
            ),
            (
                "workdir/streets-reference-gazdagret.lst",
                &ref_streets_value,
            ),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_housenumbers_value,
            ),
            ("workdir/gazdagret-streets.percent", &missing_streets_value),
            ("workdir/gazdagret.percent", &missing_housenumbers_value),
            (
                "workdir/gazdagret-additional-streets.count",
                &additional_streets_value,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = areas::Relations::new(&ctx).unwrap();

    our_main(
        &ctx,
        &mut relations,
        /*mode=*/ "relations",
        /*update=*/ true,
        /*overpass=*/ true,
    )
    .unwrap();

    // update_osm_streets() is called.
    {
        let mut guard = osm_streets_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
    // update_osm_housenumbers() is called.
    {
        let mut guard = osm_housenumbers_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
    // update_ref_streets() is called.
    {
        let mut guard = ref_streets_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
    // update_ref_housenumbers() is called.
    {
        let mut guard = ref_housenumbers_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
    // update_missing_streets() is called.
    {
        let mut guard = missing_streets_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
    // update_missing_housenumbers() is called.
    {
        let mut guard = missing_housenumbers_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
    // update_additional_streets() is called.
    {
        let mut guard = additional_streets_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
}

/// Tests our_main(): the stats case.
#[test]
fn test_our_main_stats() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let routes = vec![
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/status",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-status-happy.txt",
        ),
        context::tests::URLRoute::new(
            /*url=*/ "https://overpass-api.de/api/interpreter",
            /*data_path=*/ "",
            /*result_path=*/ "tests/network/overpass-stats.csv",
        ),
    ];
    let network = context::tests::TestNetwork::new(&routes);
    let network_arc: Arc<dyn context::Network> = Arc::new(network);
    ctx.set_network(&network_arc);
    let mut file_system = context::tests::TestFileSystem::new();
    let stats_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/stats/stats.json", &stats_value)],
    );
    file_system.set_files(&files);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();

    our_main(
        &ctx,
        &mut relations,
        /*mode=*/ "stats",
        /*update=*/ false,
        /*overpass=*/ true,
    )
    .unwrap();

    let mut guard = stats_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
}

/// Tests main().
#[test]
fn test_main() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let stats_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/stats/stats.json", &stats_value)],
    );
    file_system.set_files(&files);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let argv = vec![
        "".to_string(),
        "--mode".to_string(),
        "stats".to_string(),
        "--no-overpass".to_string(),
    ];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());

    main(&argv, &mut buf, &mut ctx).unwrap();

    // Make sure that stats.json is updated.
    let mut guard = stats_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
}

/// Tests main(): the path when our_main() returns an error.
#[test]
fn test_main_error() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let unit = context::tests::TestUnit::new();
    let unit_arc: Arc<dyn context::Unit> = Arc::new(unit);
    ctx.set_unit(&unit_arc);
    let argv = vec![
        "".to_string(),
        "--mode".to_string(),
        "stats".to_string(),
        "--no-overpass".to_string(),
    ];
    let mut buf: std::io::Cursor<Vec<u8>> = std::io::Cursor::new(Vec::new());

    // main() catches the error returned by our_main().
    main(&argv, &mut buf, &mut ctx).unwrap();
}

/// Tests update_stats_count().
#[test]
fn test_update_stats_count() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let today_csv_value = context::tests::TestFileSystem::make_file();
    today_csv_value
        .borrow_mut()
        .write_all(
            r#"addr:postcode	addr:city	addr:street	addr:housenumber	@user
7677	Orfű	Dollár utca	1	mgpx
"#
            .as_bytes(),
        )
        .unwrap();
    let today_count_value = context::tests::TestFileSystem::make_file();
    let today_citycount_value = context::tests::TestFileSystem::make_file();
    let today_zipcount_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/2020-05-10.csv", &today_csv_value),
            ("workdir/stats/2020-05-10.count", &today_count_value),
            ("workdir/stats/2020-05-10.citycount", &today_citycount_value),
            ("workdir/stats/2020-05-10.zipcount", &today_zipcount_value),
        ],
    );
    file_system.set_files(&files);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    update_stats_count(&ctx, "2020-05-10").unwrap();

    {
        let mut guard = today_count_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
    {
        let mut guard = today_citycount_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
    let mut guard = today_zipcount_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
}

/// Tests update_stats_count(): the case then the .csv is missing.
#[test]
fn test_update_stats_count_no_csv() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let today_count_value = context::tests::TestFileSystem::make_file();
    let today_citycount_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/2020-05-10.count", &today_count_value),
            ("workdir/stats/2020-05-10.citycount", &today_citycount_value),
        ],
    );
    file_system.set_files(&files);
    file_system.set_hide_paths(&[ctx.get_abspath("workdir/stats/2020-05-10.csv")]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    update_stats_count(&ctx, "2020-05-10").unwrap();

    // No .csv, no .count or .citycount.
    {
        let mut guard = today_count_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap(), 0);
    }
    {
        let mut guard = today_citycount_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap(), 0);
    }
}

/// Tests update_stats_topusers().
#[test]
fn test_update_stats_topusers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let today_csv_value = context::tests::TestFileSystem::make_file();
    today_csv_value
        .borrow_mut()
        .write_all(
            r#"addr:postcode	addr:city	addr:street	addr:housenumber	@user
7677	Orfű	Dollár utca	1	mgpx
"#
            .as_bytes(),
        )
        .unwrap();
    let today_topusers_value = context::tests::TestFileSystem::make_file();
    let today_usercount_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/2020-05-10.csv", &today_csv_value),
            ("workdir/stats/2020-05-10.topusers", &today_topusers_value),
            ("workdir/stats/2020-05-10.usercount", &today_usercount_value),
        ],
    );
    file_system.set_files(&files);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    update_stats_topusers(&ctx, "2020-05-10").unwrap();

    {
        let mut guard = today_topusers_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
    {
        let mut guard = today_usercount_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    }
}

/// Tests update_stats_topusers(): the case then the .csv is missing.
#[test]
fn test_update_stats_topusers_no_csv() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let today_topusers_value = context::tests::TestFileSystem::make_file();
    let today_usercount_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/stats/2020-05-10.topusers", &today_topusers_value),
            ("workdir/stats/2020-05-10.usercount", &today_usercount_value),
        ],
    );
    file_system.set_files(&files);
    file_system.set_hide_paths(&[ctx.get_abspath("workdir/stats/2020-05-10.csv")]);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);

    update_stats_topusers(&ctx, "2020-05-10").unwrap();

    // No .csv, no .topusers or .usercount.
    {
        let mut guard = today_topusers_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap(), 0);
    }
    {
        let mut guard = today_usercount_value.borrow_mut();
        assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap(), 0);
    }
}

/// Tests update_ref_housenumbers(): the case when we ask for CSV but get XML.
#[test]
fn test_update_ref_housenumbers_xml_as_csv() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let mut file_system = context::tests::TestFileSystem::new();
    let osm_streets_value = context::tests::TestFileSystem::make_file();
    let ref_housenumbers_value = context::tests::TestFileSystem::make_file();
    osm_streets_value
        .borrow_mut()
        .write_all(b"@id\n42\n")
        .unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/streets-gazdagret.csv", &osm_streets_value),
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_housenumbers_value,
            ),
        ],
    );
    file_system.set_files(&files);
    let file_system_arc: Arc<dyn context::FileSystem> = Arc::new(file_system);
    ctx.set_file_system(&file_system_arc);
    let mut relations = areas::Relations::new(&ctx).unwrap();
    update_ref_housenumbers(&ctx, &mut relations, /*update=*/ true).unwrap();
}
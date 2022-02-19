/*
 * Copyright 2022 Miklos Vajna. All rights reserved.
 * Use of this source code is governed by a BSD-style license that can be
 * found in the LICENSE file.
 */

#![deny(warnings)]
#![warn(clippy::all)]
#![warn(missing_docs)]

//! Tests for the areas module.

use super::*;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

/// Tests normalize().
#[test]
fn test_normalize() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let street_is_even_odd = relation.config.get_street_is_even_odd("mystreet");
    let house_numbers = normalize(
        &relation,
        "139",
        "mystreet",
        street_is_even_odd,
        &normalizers,
    )
    .unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["139"])
}

/// Tests normalize: when the number is not in range.
#[test]
fn test_normalize_not_in_range() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Budaörsi út": {
                    "ranges": [
                        {
                            "start": "1",
                            "end": "499",
                        }
                    ],
                },
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let street_is_even_odd = relation.config.get_street_is_even_odd("Budaörsi út");
    let house_numbers = normalize(
        &relation,
        "999",
        "Budaörsi út",
        street_is_even_odd,
        &normalizers,
    )
    .unwrap();
    assert_eq!(house_numbers.is_empty(), true);
}

/// Tests normalize: the case when the house number is not a number.
#[test]
fn test_normalize_not_a_number() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let street_is_even_odd = relation.get_config().get_street_is_even_odd("Budaörsi út");
    let house_numbers = normalize(
        &relation,
        "x",
        "Budaörsi út",
        street_is_even_odd,
        &normalizers,
    )
    .unwrap();
    assert_eq!(house_numbers.is_empty(), true);
}

/// Tests normalize: the case when there is no filter for this street.
#[test]
fn test_normalize_nofilter() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let street_is_even_odd = relation.get_config().get_street_is_even_odd("Budaörsi út");
    let house_numbers = normalize(
        &relation,
        "1",
        "Budaörs út",
        street_is_even_odd,
        &normalizers,
    )
    .unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["1"])
}

/// Tests normalize: the case when ';' is a separator.
#[test]
fn test_normalize_separator_semicolon() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let street_is_even_odd = relation.get_config().get_street_is_even_odd("Budaörsi út");
    let house_numbers = normalize(
        &relation,
        "1;2",
        "Budaörs út",
        street_is_even_odd,
        &normalizers,
    )
    .unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["1", "2"])
}

/// Tests normalize: the 2-6 case means implicit 4.
#[test]
fn test_normalize_separator_interval() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let street_is_even_odd = relation.get_config().get_street_is_even_odd("mystreet");
    let house_numbers = normalize(
        &relation,
        "2-6",
        "mystreet",
        street_is_even_odd,
        &normalizers,
    )
    .unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["2", "4", "6"])
}

/// Tests normalize: the 5-8 case: means just 5 and 8 as the parity doesn't match.
#[test]
fn test_normalize_separator_interval_parity() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let street_is_even_odd = relation.get_config().get_street_is_even_odd("Budaörsi út");
    let house_numbers = normalize(
        &relation,
        "5-8",
        "Budaörs út",
        street_is_even_odd,
        &normalizers,
    )
    .unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["5", "8"])
}

/// Tests normalize: the 2-5 case: means implicit 3 and 4 (interpolation=all).
#[test]
fn test_normalize_separator_interval_interp_all() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Hamzsabégi út": {
                    "interpolation": "all",
                },
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let street_is_even_odd = relation.config.get_street_is_even_odd("Hamzsabégi út");
    let house_numbers = normalize(
        &relation,
        "2-5",
        "Hamzsabégi út",
        street_is_even_odd,
        &normalizers,
    )
    .unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["2", "3", "4", "5"])
}

/// Tests normalize: the case where x-y is partially filtered out.
#[test]
fn test_normalize_separator_interval_filter() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Budaörsi út": {
                    "ranges": [
                        {
                            "start": "137",
                            "end": "165",
                        }
                    ],
                },
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let street_is_even_odd = relation.config.get_street_is_even_odd("Budaörsi út");
    // filter is 137-165
    let house_numbers = normalize(
        &relation,
        "163-167",
        "Budaörsi út",
        street_is_even_odd,
        &normalizers,
    )
    .unwrap();
    // Make sure there is no 167.
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["163", "165"])
}

/// Tests normalize: the case where x-y is nonsense: y is too large.
#[test]
fn test_normalize_separator_interval_block() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let street_is_even_odd = relation.config.get_street_is_even_odd("mystreet");
    let house_numbers = normalize(
        &relation,
        "2-2000",
        "mystreet",
        street_is_even_odd,
        &normalizers,
    )
    .unwrap();
    // Make sure that we simply ignore 2000: it's larger than the default <998 filter and the
    // 2-2000 range would be too large.
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["2"])
}

/// Tests normalize: the case where x-y is nonsense: y-x is too large.
#[test]
fn test_normalize_separator_interval_block2() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let street_is_even_odd = relation.config.get_street_is_even_odd("mystreet");
    let house_numbers = normalize(
        &relation,
        "2-56",
        "mystreet",
        street_is_even_odd,
        &normalizers,
    )
    .unwrap();
    // No expansions for 4, 6, etc.
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["2", "56"])
}

/// Tests normalize: the case where x-y is nonsense: x is 0.
#[test]
fn test_normalize_separator_interval_block3() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let street_is_even_odd = relation.config.get_street_is_even_odd("Budaörsi út");
    let house_numbers = normalize(
        &relation,
        "0-42",
        "Budaörs út",
        street_is_even_odd,
        &normalizers,
    )
    .unwrap();
    // No expansion like 0, 2, 4, etc.
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["42"])
}

/// Tests normalize: the case where x-y is only partially useful: x is OK, but y is a suffix.
#[test]
fn test_normalize_separator_interval_block4() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let street_is_even_odd = relation.config.get_street_is_even_odd("Budaörsi út");
    let house_numbers = normalize(
        &relation,
        "42-1",
        "Budaörs út",
        street_is_even_odd,
        &normalizers,
    )
    .unwrap();
    // No "1", just "42".
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["42"])
}

/// Tests normalize: the * suffix is preserved.
#[test]
fn test_normalize_keep_suffix() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let street_is_even_odd = relation.config.get_street_is_even_odd("Budaörsi út");
    let house_numbers = normalize(
        &relation,
        "1*",
        "Budaörs út",
        street_is_even_odd,
        &normalizers,
    )
    .unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["1*"]);
    let house_numbers = normalize(
        &relation,
        "2",
        "Budaörs út",
        street_is_even_odd,
        &normalizers,
    )
    .unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, vec!["2"]);
}

/// Tests normalize: the case when ',' is a separator.
#[test]
fn test_normalize_separator_comma() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let normalizers = relation.get_street_ranges().unwrap();
    let street_is_even_odd = relation.config.get_street_is_even_odd("Budaörsi út");
    let house_numbers = normalize(
        &relation,
        "2,6",
        "Budaörs út",
        street_is_even_odd,
        &normalizers,
    )
    .unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    // Same as ";", no 4.
    assert_eq!(actual, vec!["2", "6"]);
}

/// Tests Relation.get_osm_streets().
#[test]
fn test_relation_get_osm_streets() {
    let ctx = context::tests::make_test_context().unwrap();
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("test").unwrap();
    let actual: Vec<String> = relation
        .get_osm_streets(/*sorted_result=*/ true)
        .unwrap()
        .iter()
        .map(|i| i.get_osm_name().clone())
        .collect();
    let expected: Vec<String> = vec!["B1".into(), "B2".into(), "HB1".into(), "HB2".into()];
    assert_eq!(actual, expected);
}

/// Tests Relation.get_osm_streets(): when overpass gives garbage output.
#[test]
fn test_relation_get_osm_streets_bad_overpass() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let streets_value = context::tests::TestFileSystem::make_file();
    // This is garbage, it only has a single column.
    streets_value.borrow_mut().write_all(b"@id\n42\n").unwrap();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/streets-test.csv", &streets_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("test").unwrap();
    assert_eq!(
        relation.get_osm_streets(/*sorted_result=*/ true).is_err(),
        true
    );
}

/// Tests Relation.get_osm_streets(): the case when the street name is coming from a house
/// number (node).
#[test]
fn test_relation_get_osm_streets_street_is_node() {
    let ctx = context::tests::make_test_context().unwrap();
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gh830").unwrap();
    let actual = relation.get_osm_streets(/*sorted_result=*/ true).unwrap();
    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].get_osm_type(), "node");
}

/// Tests Relation.get_osm_streets(): the case when we have streets, but no house numbers.
#[test]
fn test_relation_get_osm_streets_no_house_number() {
    let ctx = context::tests::make_test_context().unwrap();
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("ujbuda").unwrap();
    let osm_streets = relation.get_osm_streets(/*sorted_result=*/ true).unwrap();
    let actual: Vec<_> = osm_streets.iter().map(|i| i.get_osm_name()).collect();
    let expected = vec!["OSM Name 1", "Törökugrató utca", "Tűzkő utca"];
    assert_eq!(actual, expected);
}

/// Tests Relation.get_osm_streets(): when there is only an addr:conscriptionnumber.
#[test]
fn test_relation_get_osm_streets_conscriptionnumber() {
    let ctx = context::tests::make_test_context().unwrap();
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gh754").unwrap();
    let osm_streets = relation.get_osm_streets(/*sorted_result=*/ true).unwrap();
    let streets: Vec<_> = osm_streets.iter().map(|i| i.get_osm_name()).collect();
    // This is coming from a house number which has addr:street and addr:conscriptionnumber, but
    // no addr:housenumber.
    let expected: &String = &String::from("Barcfa dűlő");
    assert_eq!(streets.contains(&expected), true);
}

/// Tests Relation.get_osm_streets_query().
#[test]
fn test_relation_get_osm_streets_query() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let relation = relations.get_relation(relation_name).unwrap();
    let ret = relation.get_osm_streets_query().unwrap();
    assert_eq!(ret, "aaa 42 bbb 3600000042 ccc\n");
}

/// Tests Relation.get_osm_housenumbers_query().
#[test]
fn test_relation_get_osm_housenumbers_query() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let ret = relation.get_osm_housenumbers_query().unwrap();
    assert_eq!(ret, "housenr aaa 42 bbb 3600000042 ccc\n");
}

/// Tests RelationFiles.write_osm_streets().
#[test]
fn test_relation_files_write_osm_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let streets_value = context::tests::TestFileSystem::make_file();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/streets-gazdagret.csv", &streets_value),
            ("data/yamls.cache", &yamls_cache_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let relation = relations.get_relation(relation_name).unwrap();
    let result_from_overpass =
        "@id\tname\n1\tTűzkő utca\n2\tTörökugrató utca\n3\tOSM Name 1\n4\tHamzsabégi út\n";
    let expected = std::fs::read("tests/workdir/streets-gazdagret.csv").unwrap();
    relation
        .get_files()
        .write_osm_streets(&ctx, result_from_overpass)
        .unwrap();
    let mut guard = streets_value.borrow_mut();
    guard.seek(SeekFrom::Start(0)).unwrap();
    let mut actual: Vec<u8> = Vec::new();
    guard.read_to_end(&mut actual).unwrap();
    assert_eq!(actual, expected);
}

/// Tests RelationFiles.write_osm_housenumbers().
#[test]
fn test_relation_files_write_osm_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let housenumbers_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            (
                "workdir/street-housenumbers-gazdagret.csv",
                &housenumbers_value,
            ),
            ("data/yamls.cache", &yamls_cache_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let result_from_overpass =
        "@id\taddr:street\taddr:housenumber\taddr:postcode\taddr:housename\t\
addr:conscriptionnumber\taddr:flats\taddr:floor\taddr:door\taddr:unit\tname\t@type\n\n\
1\tTörökugrató utca\t1\t\t\t\t\t\t\t\t\tnode\n\
1\tTörökugrató utca\t2\t\t\t\t\t\t\t\t\tnode\n\
1\tTűzkő utca\t9\t\t\t\t\t\t\t\t\tnode\n\
1\tTűzkő utca\t10\t\t\t\t\t\t\t\t\tnode\n\
1\tOSM Name 1\t1\t\t\t\t\t\t\t\t\tnode\n\
1\tOSM Name 1\t2\t\t\t\t\t\t\t\t\tnode\n\
1\tOnly In OSM utca\t1\t\t\t\t\t\t\t\t\tnode\n\
1\tSecond Only In OSM utca\t1\t\t\t\t\t\t\t\t\tnode\n";
    let expected = String::from_utf8(
        std::fs::read("tests/workdir/street-housenumbers-gazdagret.csv").unwrap(),
    )
    .unwrap();
    let relation = relations.get_relation(relation_name).unwrap();
    relation
        .get_files()
        .write_osm_housenumbers(&ctx, result_from_overpass)
        .unwrap();
    let mut guard = housenumbers_value.borrow_mut();
    guard.seek(SeekFrom::Start(0)).unwrap();
    let mut actual: Vec<u8> = Vec::new();
    guard.read_to_end(&mut actual).unwrap();
    assert_eq!(String::from_utf8(actual).unwrap(), expected);
}

/// Tests Relation::get_street_ranges().
#[test]
fn test_relation_get_street_ranges() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relation-myrelation.yaml": {
            "filters": {
                "mystreet1": {
                    "ranges": [
                        {
                            "start": "1",
                            "end": "3",
                        },
                    ],
                },
            },
            "refstreets": {
                "myosm": "myref",
            },
            "street-filters": [
                "mystreet2",
            ],
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let filters = relation.get_street_ranges().unwrap();
    let mut expected_filters: HashMap<String, ranges::Ranges> = HashMap::new();
    expected_filters.insert(
        "mystreet1".into(),
        ranges::Ranges::new(vec![ranges::Range::new(1, 3, "")]),
    );
    assert_eq!(filters, expected_filters);

    let mut expected_streets: HashMap<String, String> = HashMap::new();
    expected_streets.insert("myosm".into(), "myref".into());
    assert_eq!(relation.get_config().get_refstreets(), expected_streets);

    let street_blacklist = relation.get_config().get_street_filters();
    assert_eq!(street_blacklist, ["mystreet2".to_string()]);
}

/// Tests Relation::get_street_ranges(): when the filter file is empty.
#[test]
fn test_relation_get_street_ranges_empty() {
    let ctx = context::tests::make_test_context().unwrap();
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("empty").unwrap();
    let filters = relation.get_street_ranges().unwrap();
    assert_eq!(filters.is_empty(), true);
}

/// Tests Relation::get_ref_street_from_osm_street().
#[test]
fn test_relation_get_ref_street_from_osm_street() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let mut street: String = "mystreet".into();
    let relation_name = "myrelation";
    let relation = relations.get_relation(relation_name).unwrap();
    let refcounty = relation.get_config().get_refcounty();
    street = relation
        .get_config()
        .get_ref_street_from_osm_street(&street);
    assert_eq!(refcounty, "01");
    assert_eq!(
        relation.get_config().get_street_refsettlement(&street),
        ["011"]
    );
    assert_eq!(street, "mystreet");
}

/// Tests Relation::get_ref_street_from_osm_street(): street-specific refsettlement override.
#[test]
fn test_relation_get_ref_street_from_osm_street_refsettlement_override() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-myrelation.yaml": {
            "filters": {
                "mystreet": {
                    // this would be 011 by default, but here it's overwritten at a street
                    // level
                    "refsettlement": "012",
                }
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
    let mut relations = Relations::new(&ctx).unwrap();
    let street = "mystreet";
    let relation_name = "myrelation";
    let relation = relations.get_relation(relation_name).unwrap();
    let refcounty = relation.get_config().get_refcounty();
    let street = relation.get_config().get_ref_street_from_osm_street(street);
    assert_eq!(refcounty, "01");
    assert_eq!(
        relation.get_config().get_street_refsettlement(&street),
        ["012"]
    );
    assert_eq!(street, "mystreet");
}

/// Tests Relation.get_ref_street_from_osm_street(): OSM -> ref name mapping.
#[test]
fn test_relation_get_ref_street_from_osm_street_refstreets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-myrelation.yaml": {
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
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
    let mut relations = Relations::new(&ctx).unwrap();
    let street = "OSM Name 1";
    let relation_name = "myrelation";
    let relation = relations.get_relation(relation_name).unwrap();
    let refcounty = relation.get_config().get_refcounty();
    let street = relation
        .get_config()
        .get_ref_street_from_osm_street(&street);
    assert_eq!(refcounty, "01");
    assert_eq!(
        relation.get_config().get_street_refsettlement(&street),
        ["011"]
    );
    assert_eq!(street, "Ref Name 1");
}

/// Tests Relation.get_ref_street_from_osm_street(): a relation with an empty filter file.
#[test]
fn test_relation_get_ref_street_from_osm_street_emptyrelation() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let street = "OSM Name 1";
    let relation_name = "myrelation";
    let relation = relations.get_relation(relation_name).unwrap();
    let refcounty = relation.get_config().get_refcounty();
    let street = relation.get_config().get_ref_street_from_osm_street(street);
    assert_eq!(refcounty, "01");
    assert_eq!(
        relation.get_config().get_street_refsettlement(&street),
        ["011"]
    );
    assert_eq!(street, "OSM Name 1");
}

/// Tests Relation.get_ref_street_from_osm_street(): the refsettlement range-level override.
#[test]
fn test_relation_get_ref_street_from_osm_street_range_level_override() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-myrelation.yaml": {
            "filters": {
                "mystreet": {
                    "ranges": [
                    {
                        "start": "1",
                        "end": "1",
                        "refsettlement": "013",
                    },
                    ]
                },
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
    let mut relations = Relations::new(&ctx).unwrap();
    let street = "mystreet";
    let relation_name = "myrelation";
    let relation = relations.get_relation(relation_name).unwrap();
    let refcounty = relation.get_config().get_refcounty();
    let street = relation.get_config().get_ref_street_from_osm_street(street);
    assert_eq!(refcounty, "01");
    assert_eq!(
        relation.get_config().get_street_refsettlement(&street),
        ["011", "013"]
    );
    assert_eq!(street, "mystreet");
}

/// Tests make_turbo_query_for_streets().
#[test]
fn test_make_turbo_query_for_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 2713748,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    let from = ["A2".to_string()];
    let ret = make_turbo_query_for_streets(&relation, &from);
    let expected = r#"[out:json][timeout:425];
rel(2713748)->.searchRelation;
area(3602713748)->.searchArea;
(rel(2713748);
way["name"="A2"](r.searchRelation);
way["name"="A2"](area.searchArea);
);
out body;
>;
out skel qt;
{{style:
relation{width:3}
way{color:blue; width:4;}
}}"#;
    assert_eq!(ret, expected);
}

/// Tests Relation::get_ref_streets().
#[test]
fn test_relation_get_ref_streets() {
    let ctx = context::tests::make_test_context().unwrap();
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let relation = relations.get_relation(relation_name).unwrap();
    let streets = relation.get_ref_streets().unwrap();
    assert_eq!(
        streets,
        [
            "Hamzsabégi út",
            "Only In Ref Nonsense utca",
            "Only In Ref utca",
            "Ref Name 1",
            "Törökugrató utca",
            "Tűzkő utca"
        ]
    );
}

/// Tests Relation::get_osm_housenumbers().
#[test]
fn test_relation_get_osm_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let street_name = "Törökugrató utca";
    let mut relation = relations.get_relation(relation_name).unwrap();
    let house_numbers = relation.get_osm_housenumbers(street_name).unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, ["1", "2"]);
}

/// Tests Relation::get_osm_housenumbers(): the case when addr:place is used instead of addr:street.
#[test]
fn test_relation_get_osm_housenumbers_addr_place() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gh964";
    let mut relation = relations.get_relation(relation_name).unwrap();
    let street_name = "Tolvajos tanya";
    let house_numbers = relation.get_osm_housenumbers(street_name).unwrap();
    let actual: Vec<_> = house_numbers.iter().map(|i| i.get_number()).collect();
    assert_eq!(actual, ["52"]);
}

/// Tests Relation::get_missing_housenumbers().
#[test]
fn test_relation_get_missing_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Törökugrató utca": {
                    "invalid": [ "11", "12" ],
                }
            },
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let mut relation = relations.get_relation(relation_name).unwrap();
    let (ongoing_streets, done_streets) = relation.get_missing_housenumbers().unwrap();
    let ongoing_streets_strs: Vec<_> = ongoing_streets
        .iter()
        .map(|(name, numbers)| {
            let numbers: Vec<_> = numbers.iter().map(|i| i.get_number()).collect();
            (name.get_osm_name().clone(), numbers)
        })
        .collect();
    // Notice how 11 and 12 is filtered out by the 'invalid' mechanism for 'Törökugrató utca'.
    assert_eq!(
        ongoing_streets_strs,
        [
            ("Törökugrató utca".to_string(), vec!["7", "10"]),
            ("Tűzkő utca".to_string(), vec!["1", "2"]),
            ("Hamzsabégi út".to_string(), vec!["1"])
        ]
    );
    let expected = [
        ("OSM Name 1".to_string(), vec!["1", "2"]),
        ("Törökugrató utca".to_string(), vec!["1", "2"]),
        ("Tűzkő utca".to_string(), vec!["9", "10"]),
    ];
    let done_streets_strs: Vec<_> = done_streets
        .iter()
        .map(|(name, numbers)| {
            let numbers: Vec<_> = numbers.iter().map(|i| i.get_number()).collect();
            (name.get_osm_name().clone(), numbers)
        })
        .collect();
    assert_eq!(done_streets_strs, expected);
}

/// Sets the housenumber_letters property from code.
fn set_config_housenumber_letters(config: &mut RelationConfig, housenumber_letters: bool) {
    config.dict.housenumber_letters = Some(housenumber_letters);
}

/// Sets the 'filters' key from code.
fn set_config_filters(config: &mut RelationConfig, filters: &HashMap<String, RelationFiltersDict>) {
    config.dict.filters = Some(filters.clone());
}

/// Tests Relation::get_missing_housenumbers(): 7/A is detected when 7/B is already mapped.
#[test]
fn test_relation_get_missing_housenumbers_letter_suffix() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gh267": {
                "osmrelation": 42,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gh267";
    let mut relation = relations.get_relation(relation_name).unwrap();
    // Opt-in, this is not the default behavior.
    let mut config = relation.get_config().clone();
    set_config_housenumber_letters(&mut config, true);
    relation.set_config(&config);
    let (ongoing_streets, _done_streets) = relation.get_missing_housenumbers().unwrap();
    let ongoing_street = ongoing_streets[0].clone();
    let housenumber_ranges = util::get_housenumber_ranges(&ongoing_street.1);
    let mut housenumber_range_names: Vec<_> =
        housenumber_ranges.iter().map(|i| i.get_number()).collect();
    housenumber_range_names.sort_by_key(|i| util::split_house_number(i));
    // Make sure that 1/1 shows up in the output: it's not the same as '1' or '11'.
    let expected = [
        "1", "1/1", "1/2", "3", "5", "7", "7/A", "7/B", "7/C", "9", "11", "13", "13-15",
    ];
    assert_eq!(housenumber_range_names, expected);
}

/// Tests Relation::get_missing_housenumbers(): how 'invalid' interacts with normalization.
#[test]
fn test_relation_get_missing_housenumbers_letter_suffix_invalid() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gh296": {
                "osmrelation": 42,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gh296";
    let mut relation = relations.get_relation(relation_name).unwrap();
    // Opt-in, this is not the default behavior.
    let mut config = relation.get_config().clone();
    set_config_housenumber_letters(&mut config, true);
    // Set custom 'invalid' map.
    let filters: HashMap<String, RelationFiltersDict> = serde_json::from_value(serde_json::json!({
        "Rétköz utca": {
            "invalid": ["9", "47"]
        }
    }))
    .unwrap();
    set_config_filters(&mut config, &filters);
    relation.set_config(&config);
    let (ongoing_streets, _) = relation.get_missing_housenumbers().unwrap();
    let ongoing_street = ongoing_streets[0].clone();
    let housenumber_ranges = util::get_housenumber_ranges(&ongoing_street.1);
    let mut housenumber_range_names: Vec<_> =
        housenumber_ranges.iter().map(|i| i.get_number()).collect();
    housenumber_range_names.sort_by_key(|i| util::split_house_number(i));
    // Notice how '9 A 1' is missing here: it's not a simple house number, so it gets normalized
    // to just '9' and the above filter silences it.
    let expected = ["9/A"];
    assert_eq!(housenumber_range_names, expected);
}

/// Tests Relation::get_missing_housenumbers(): how 'invalid' interacts with housenumber-letters: true or false.
#[test]
fn test_relation_get_missing_housenumbers_invalid_simplify() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gh385": {
                "osmrelation": 42,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gh385";
    let mut relation = relations.get_relation(relation_name).unwrap();

    // Default case: housenumber-letters=false.
    {
        let filters: HashMap<String, RelationFiltersDict> =
            serde_json::from_value(serde_json::json!({
                "Kővirág sor": {
                    "invalid": ["37b"]
                }
            }))
            .unwrap();
        let mut config = relation.get_config().clone();
        set_config_filters(&mut config, &filters);
        relation.set_config(&config);
        let (ongoing_streets, _) = relation.get_missing_housenumbers().unwrap();
        // Note how 37b from invalid is simplified to 37; and how 37/B from ref is simplified to
        // 37 as well, so we find the match.
        assert_eq!(ongoing_streets.is_empty(), true);
    }

    // Opt-in case: housenumber-letters=true.
    {
        let mut config = relation.get_config().clone();
        set_config_housenumber_letters(&mut config, true);
        relation.set_config(&config);
        let filters: HashMap<String, RelationFiltersDict> =
            serde_json::from_value(serde_json::json!({
                "Kővirág sor": {
                    "invalid": ["37b"]
                }
            }))
            .unwrap();
        set_config_filters(&mut config, &filters);
        relation.set_config(&config);
        let (ongoing_streets, _) = relation.get_missing_housenumbers().unwrap();
        // In this case 37b from invalid matches 37/B from ref.
        assert_eq!(ongoing_streets.is_empty(), true);
    }

    // Make sure out-of-range invalid elements are just ignored and no exception is raised.
    let mut config = relation.get_config().clone();
    set_config_housenumber_letters(&mut config, true);
    relation.set_config(&config);
    let filters: HashMap<String, RelationFiltersDict> = serde_json::from_value(serde_json::json!({
        "Kővirág sor": {
            "invalid": ["5"],
            "ranges": [{"start": "1", "end": "3"}],
        }
    }))
    .unwrap();
    set_config_filters(&mut config, &filters);
    relation.set_config(&config);
    relation.get_missing_housenumbers().unwrap();
}

/// Tests Relation::get_missing_housenumbers(): '42 A' vs '42/A' is recognized as a match.
#[test]
fn test_relation_get_missing_housenumbers_letter_suffix_normalize() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gh286": {
                "osmrelation": 42,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gh286";
    let mut relation = relations.get_relation(relation_name).unwrap();
    // Opt-in, this is not the default behavior.
    let mut config = relation.get_config().clone();
    set_config_housenumber_letters(&mut config, true);
    relation.set_config(&config);
    let (ongoing_streets, _) = relation.get_missing_housenumbers().unwrap();
    let ongoing_street = ongoing_streets[0].clone();
    let housenumber_ranges = util::get_housenumber_ranges(&ongoing_street.1);
    let mut housenumber_range_names: Vec<_> =
        housenumber_ranges.iter().map(|i| i.get_number()).collect();
    housenumber_range_names.sort_by_key(|i| util::split_house_number(i));
    // Note how 10/B is not in this list.
    let expected = ["10/A"];
    assert_eq!(housenumber_range_names, expected);
}

/// Tests Relation::get_missing_housenumbers(): '42/A*' and '42/a' matches.
#[test]
fn test_relation_get_missing_housenumbers_letter_suffix_source_suffix() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gh299": {
                "osmrelation": 42,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gh299";
    let mut relation = relations.get_relation(relation_name).unwrap();
    // Opt-in, this is not the default behavior.
    let mut config = relation.get_config().clone();
    set_config_housenumber_letters(&mut config, true);
    relation.set_config(&config);
    let (ongoing_streets, _) = relation.get_missing_housenumbers().unwrap();
    // Note how '52/B*' is not in this list.
    assert_eq!(ongoing_streets, []);
}

/// Tests Relation::get_missing_housenumbers(): 'a' is not stripped from '1;3a'.
#[test]
fn test_relation_get_missing_housenumbers_letter_suffix_normalize_semicolon() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gh303": {
                "osmrelation": 42,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gh303";
    let mut relation = relations.get_relation(relation_name).unwrap();
    // Opt-in, this is not the default behavior.
    let mut config = relation.get_config().clone();
    set_config_housenumber_letters(&mut config, true);
    relation.set_config(&config);
    let (ongoing_streets, _) = relation.get_missing_housenumbers().unwrap();
    let ongoing_street = ongoing_streets[0].clone();
    let housenumber_ranges = util::get_housenumber_ranges(&ongoing_street.1);
    let mut housenumber_range_names: Vec<_> =
        housenumber_ranges.iter().map(|i| i.get_number()).collect();
    housenumber_range_names.sort_by_key(|i| util::split_house_number(i));
    // Note how 43/B and 43/C is not here.
    let expected = ["43/A", "43/D"];
    assert_eq!(housenumber_range_names, expected);
}

/// Tests Relation::get_missing_streets().
#[test]
fn test_relation_get_missing_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
        "relation-gazdagret.yaml": {
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            },
            "street-filters": [
                "Only In Ref Nonsense utca",
            ],
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let relation = relations.get_relation(relation_name).unwrap();
    let (only_in_reference, in_both) = relation.get_missing_streets().unwrap();

    // Note that 'Only In Ref Nonsense utca' is missing from this list.
    assert_eq!(only_in_reference, ["Only In Ref utca"]);

    assert_eq!(
        in_both,
        [
            "Hamzsabégi út",
            "Ref Name 1",
            "Törökugrató utca",
            "Tűzkő utca"
        ]
    );
}

/// Tests Relation::get_additional_streets().
#[test]
fn test_relation_get_additional_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
        "relation-gazdagret.yaml": {
            "osm-street-filters": [
                "Second Only In OSM utca",
            ],
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let relation = relations.get_relation(relation_name).unwrap();
    let only_in_osm = relation
        .get_additional_streets(/*sorted_result=*/ true)
        .unwrap();

    assert_eq!(only_in_osm, [util::Street::from_string("Only In OSM utca")]);

    // These is filtered out, even if it's OSM-only.
    let osm_street_blacklist = relation.get_config().get_osm_street_filters();
    assert_eq!(osm_street_blacklist, ["Second Only In OSM utca"]);
}

/// Tests Relation::get_additional_streets(): when the osm-street-filters key is missing.
#[test]
fn test_relation_get_additional_streets_no_osm_street_filters() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gh385": {
                "osmrelation": 42,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gh385";
    let relation = relations.get_relation(relation_name).unwrap();
    assert_eq!(
        relation.get_config().get_osm_street_filters().is_empty(),
        true
    );
}

/// Relation::get_additional_housenumbers().
#[test]
fn test_relation_get_additional_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Second Only In OSM utca": {
                    "valid": ['1'],
                },
            },
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            }
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let mut relation = relations.get_relation(relation_name).unwrap();
    let only_in_osm = relation.get_additional_housenumbers().unwrap();
    let only_in_osm_strs: Vec<_> = only_in_osm
        .iter()
        .map(|(name, numbers)| {
            let numbers: Vec<_> = numbers.iter().map(|i| i.get_number()).collect();
            (name.get_osm_name(), numbers)
        })
        .collect();
    // Note how Second Only In OSM utca 1 is filtered out explicitly.
    assert_eq!(
        only_in_osm_strs,
        [(&"Only In OSM utca".to_string(), vec!["1"])]
    );
}

/// Unwraps an escaped matrix of rust.PyDocs into a string matrix.
fn table_doc_to_string(table: &[Vec<yattag::Doc>]) -> Vec<Vec<String>> {
    let mut table_content = Vec::new();
    for row in table {
        let mut row_content = Vec::new();
        for cell in row {
            row_content.push(cell.get_value());
        }
        table_content.push(row_content);
    }
    table_content
}

/// Tests Relation::write_missing_housenumbers().
#[test]
fn test_relation_write_missing_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let percent_value = context::tests::TestFileSystem::make_file();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Törökugrató utca": {
                    "invalid": ["11", "12"],
                },
            },
            "street-filters": ["Only In Ref Nonsense utca"],
            "osm-street-filters": ["Second Only In OSM utca"],
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            }
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/gazdagret.percent", &percent_value),
            ("data/yamls.cache", &yamls_cache_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let mut relation = relations.get_relation(relation_name).unwrap();

    let ret = relation.write_missing_housenumbers().unwrap();

    let (todo_street_count, todo_count, done_count, percent, table) = ret;
    assert_eq!(todo_street_count, 3);
    assert_eq!(todo_count, 5);
    assert_eq!(done_count, 6);
    assert_eq!(format!("{0:.2}", percent), "54.55");
    let string_table = table_doc_to_string(&table);
    assert_eq!(
        string_table,
        [
            ["Street name", "Missing count", "House numbers"],
            ["Törökugrató utca", "2", "7<br />10"],
            ["Tűzkő utca", "2", "1<br />2"],
            ["Hamzsabégi út", "1", "1"]
        ]
    );
    let mut guard = percent_value.borrow_mut();
    guard.seek(SeekFrom::Start(0)).unwrap();
    let mut actual: Vec<u8> = Vec::new();
    guard.read_to_end(&mut actual).unwrap();
    assert_eq!(String::from_utf8(actual).unwrap(), "54.55");
}

/// Tests Relation::write_missing_housenumbers(): the case when percent can't be determined.
#[test]
fn test_relation_write_missing_housenumbers_empty() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let percent_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/empty.percent", &percent_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "empty";
    let mut relation = relations.get_relation(relation_name).unwrap();

    let ret = relation.write_missing_housenumbers().unwrap();

    let (_todo_street_count, _todo_count, _done_count, percent, _table) = ret;
    assert_eq!(percent, 100.0);
    assert_eq!(relation.config.get_filters().is_none(), true);
}

/// Tests Relation::write_missing_housenumbers(): the case when the street is interpolation=all and coloring is wanted.
#[test]
fn test_relation_write_missing_housenumbers_interpolation_all() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let percent_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/budafok.percent", &percent_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "budafok";
    let mut relation = relations.get_relation(relation_name).unwrap();

    let ret = relation.write_missing_housenumbers().unwrap();

    let (_todo_street_count, _todo_count, _done_count, _percent, table) = ret;
    let string_table = table_doc_to_string(&table);
    assert_eq!(
        string_table,
        [
            ["Street name", "Missing count", "House numbers"],
            [
                "Vöröskúti határsor",
                "4",
                "2, 12, 34, <span style=\"color: blue;\">36</span>"
            ]
        ]
    );
    let mut guard = percent_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
}

/// Tests Relation::write_missing_housenumbers(): sorting is performed after range reduction.
#[test]
fn test_relation_write_missing_housenumbers_sorting() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let percent_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/gh414.percent", &percent_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gh414";
    let mut relation = relations.get_relation(relation_name).unwrap();

    let ret = relation.write_missing_housenumbers().unwrap();

    let (_todo_street_count, _todo_count, _done_count, _percent, table) = ret;
    let string_table = table_doc_to_string(&table);
    // Note how 'A utca' is logically 5 house numbers, but it's a single range, so it's
    // ordered after 'B utca'.
    assert_eq!(
        string_table,
        [
            ["Street name", "Missing count", "House numbers"],
            ["B utca", "2", "1, 3"],
            ["A utca", "1", "2-10"]
        ]
    );
    let mut guard = percent_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
}

/// Tests Relation::write_missing_streets().
#[test]
fn test_write_missing_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-gazdagret.yaml": {
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            },
            "street-filters": ["Only In Ref Nonsense utca"],
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let percent_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/gazdagret-streets.percent", &percent_value),
            ("data/yamls.cache", &yamls_cache_value),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let relation = relations.get_relation(relation_name).unwrap();
    let expected = String::from_utf8(
        std::fs::read(&ctx.get_abspath("workdir/gazdagret-streets.percent")).unwrap(),
    )
    .unwrap();

    let ret = relation.write_missing_streets().unwrap();

    let (todo_count, done_count, percent, streets) = ret;

    assert_eq!(todo_count, 1);
    assert_eq!(done_count, 4);
    assert_eq!(format!("{0:.2}", percent), "80.00");
    assert_eq!(streets, ["Only In Ref utca"]);
    let mut guard = percent_value.borrow_mut();
    guard.seek(SeekFrom::Start(0)).unwrap();
    let mut actual: Vec<u8> = Vec::new();
    guard.read_to_end(&mut actual).unwrap();
    assert_eq!(String::from_utf8(actual).unwrap(), expected);
}

/// Tests Relation::write_missing_streets(): the case when percent can't be determined.
#[test]
fn test_write_missing_streets_empty() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let percent_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("workdir/empty-streets.percent", &percent_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "empty";
    let relation = relations.get_relation(relation_name).unwrap();

    let ret = relation.write_missing_streets().unwrap();

    let mut guard = percent_value.borrow_mut();
    assert_eq!(guard.seek(SeekFrom::Current(0)).unwrap() > 0, true);
    let (_todo_count, _done_count, percent, _streets) = ret;
    assert_eq!(format!("{0:.2}", percent), "100.00");
}

/// Tests Relation::build_ref_housenumbers().
#[test]
fn test_relation_build_ref_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_housenumbers_cache = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "refdir/hazszamok_20190511.tsv-01-v1.cache",
                &ref_housenumbers_cache,
            ),
        ],
    );
    let refdir = ctx.get_abspath("refdir");
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let refpath = format!("{}/hazszamok_20190511.tsv", refdir);
    let memory_cache = util::build_reference_cache(&ctx, &refpath, "01").unwrap();
    let relation_name = "myrelation";
    let street = "Törökugrató utca";
    let relation = relations.get_relation(relation_name).unwrap();
    let ret = relation.build_ref_housenumbers(&memory_cache, street, "");
    let expected = [
        "Törökugrató utca\t1\t",
        "Törökugrató utca\t10\t",
        "Törökugrató utca\t11\t",
        "Törökugrató utca\t12\t",
        "Törökugrató utca\t2\t",
        "Törökugrató utca\t7\t",
    ];
    assert_eq!(ret, expected);
}

/// Tests Relation::build_ref_housenumbers(): the case when the street is not in the reference.
#[test]
fn test_relation_build_ref_housenumbers_missing() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "refsettlement": "42",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_housenumbers_cache = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            (
                "refdir/hazszamok_20190511.tsv-01-v1.cache",
                &ref_housenumbers_cache,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let refdir = ctx.get_abspath("refdir");
    let refpath = format!("{}/hazszamok_20190511.tsv", refdir);
    let memory_cache = util::build_reference_cache(&ctx, &refpath, "01").unwrap();
    let relation_name = "myrelation";
    let street = "mystreet";
    let relation = relations.get_relation(relation_name).unwrap();

    let ret = relation.build_ref_housenumbers(&memory_cache, street, "");

    assert_eq!(ret.is_empty(), true);
}

/// Tests Relation::build_ref_streets().
#[test]
fn test_relation_build_ref_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "refsettlement": "42",
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_streets_cache = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("data/yamls.cache", &yamls_cache_value),
            ("refdir/utcak_20190514.tsv.cache", &ref_streets_cache),
        ],
    );
    let refdir = ctx.get_abspath("refdir");
    let refpath = format!("{}/utcak_20190514.tsv", refdir);
    let memory_cache = util::build_street_reference_cache(&ctx, &refpath).unwrap();
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "myrelation";
    let relation = relations.get_relation(relation_name).unwrap();

    let ret = relation.config.get_ref_streets(&memory_cache);

    assert_eq!(
        ret,
        &[
            "Törökugrató utca",
            "Tűzkő utca",
            "Ref Name 1",
            "Only In Ref utca",
            "Only In Ref Nonsense utca",
            "Hamzsabégi út"
        ]
    );
}

/// Tests Relation::write_ref_housenumbers().
#[test]
fn test_relation_writer_ref_housenumbers() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
        "relation-gazdagret.yaml": {
            "refstreets": {
                "OSM Name 1": "Ref Name 1",
            }
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_housenumbers_cache = context::tests::TestFileSystem::make_file();
    let ref_housenumbers2_cache = context::tests::TestFileSystem::make_file();
    let refdir = ctx.get_abspath("refdir");
    let refpath = format!("{}/hazszamok_20190511.tsv", refdir);
    let refpath2 = format!("{}/hazszamok_kieg_20190808.tsv", refdir);
    let ref_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            (
                "workdir/street-housenumbers-reference-gazdagret.lst",
                &ref_value,
            ),
            ("data/yamls.cache", &yamls_cache_value),
            (
                "refdir/hazszamok_20190511.tsv-01-v1.cache",
                &ref_housenumbers_cache,
            ),
            (
                "refdir/hazszamok_kieg_20190808.tsv-01-v1.cache",
                &ref_housenumbers2_cache,
            ),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let expected = String::from_utf8(
        std::fs::read(&ctx.get_abspath("workdir/street-housenumbers-reference-gazdagret.lst"))
            .unwrap(),
    )
    .unwrap();
    let relation = relations.get_relation(relation_name).unwrap();

    relation
        .write_ref_housenumbers(&[refpath, refpath2])
        .unwrap();

    let mut guard = ref_value.borrow_mut();
    guard.seek(SeekFrom::Start(0)).unwrap();
    let mut actual: Vec<u8> = Vec::new();
    guard.read_to_end(&mut actual).unwrap();
    assert_eq!(String::from_utf8(actual).unwrap(), expected);
}

/// Tests Relation::write_ref_housenumbers(): the case when the refcounty code is missing in the reference.
#[test]
fn test_relation_writer_ref_housenumbers_nosuchrefcounty() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "nosuchrefcounty": {
                "refsettlement": "43",
                "refcounty": "98",
                "refsettlement": "99",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_streets_cache = context::tests::TestFileSystem::make_file();
    let ref_hns_cache = context::tests::TestFileSystem::make_file();
    let refdir = ctx.get_abspath("refdir");
    let refpath = format!("{}/hazszamok_20190511.tsv", refdir);
    let ref_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            (
                "workdir/street-housenumbers-reference-nosuchrefcounty.lst",
                &ref_value,
            ),
            ("data/yamls.cache", &yamls_cache_value),
            ("refdir/utcak_20190514.tsv.cache", &ref_streets_cache),
            ("refdir/hazszamok_20190511.tsv-98-v1.cache", &ref_hns_cache),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "nosuchrefcounty";
    let relation = relations.get_relation(relation_name).unwrap();

    relation.write_ref_housenumbers(&[refpath]).unwrap();
}

/// Tests Relation::write_ref_housenumbers(): the case when the refsettlement code is missing in the reference.
#[test]
fn test_relation_writer_ref_housenumbers_nosuchrefsettlement() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "nosuchrefsettlement": {
                "refcounty": "01",
                "refsettlement": "99",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_streets_cache = context::tests::TestFileSystem::make_file();
    let ref_hns_cache = context::tests::TestFileSystem::make_file();
    let refdir = ctx.get_abspath("refdir");
    let refpath = format!("{}/hazszamok_20190511.tsv", refdir);
    let ref_value = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            (
                "workdir/street-housenumbers-reference-nosuchrefsettlement.lst",
                &ref_value,
            ),
            ("data/yamls.cache", &yamls_cache_value),
            ("refdir/utcak_20190514.tsv.cache", &ref_streets_cache),
            ("refdir/hazszamok_20190511.tsv-01-v1.cache", &ref_hns_cache),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "nosuchrefsettlement";
    let relation = relations.get_relation(relation_name).unwrap();

    relation.write_ref_housenumbers(&[refpath]).unwrap();
}

/// Tests Relation::write_ref_streets().
#[test]
fn test_relation_write_ref_streets() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let ref_value = context::tests::TestFileSystem::make_file();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let ref_streets_cache = context::tests::TestFileSystem::make_file();
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[
            ("workdir/streets-reference-gazdagret.lst", &ref_value),
            ("data/yamls.cache", &yamls_cache_value),
            ("refdir/utcak_20190514.tsv.cache", &ref_streets_cache),
        ],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let refdir = ctx.get_abspath("refdir");
    let refpath = format!("{}/utcak_20190514.tsv", refdir);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation_name = "gazdagret";
    let relation = relations.get_relation(relation_name).unwrap();
    let expected = String::from_utf8(
        std::fs::read(&ctx.get_abspath("workdir/streets-reference-gazdagret.lst")).unwrap(),
    )
    .unwrap();

    relation.write_ref_streets(&refpath).unwrap();

    let mut guard = ref_value.borrow_mut();
    guard.seek(SeekFrom::Start(0)).unwrap();
    let mut actual: Vec<u8> = Vec::new();
    guard.read_to_end(&mut actual).unwrap();
    assert_eq!(String::from_utf8(actual).unwrap(), expected);
}

/// Tests the Relations struct.
#[test]
fn test_relations() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation1": {
                "osmrelation": 42,
                "refcounty": "01",
                "refsettlement": "011",
            },
            "myrelation2": {
                "osmrelation": 43,
                "refcounty": "43", // not 01
                "refsettlement": "011",
            },
            "myrelation3": {
                "osmrelation": 44,
                "refcounty": "01",
                "refsettlement": "99", // not 011
            },
        },
        "relation-myrelation2.yaml": {
            "inactive": true,
        },
        "relation-myrelation3.yaml": {
            "missing-streets": "only",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let expected_relation_names = ["myrelation1", "myrelation2", "myrelation3"];
    assert_eq!(relations.get_names(), expected_relation_names);
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation2".to_string()),
        false
    );
    let mut osmids: Vec<_> = relations
        .get_relations()
        .unwrap()
        .iter()
        .map(|relation| relation.get_config().get_osmrelation())
        .collect();
    osmids.sort();
    assert_eq!(osmids, [42, 43, 44]);
    let ujbuda = relations.get_relation("myrelation3").unwrap();
    assert_eq!(ujbuda.get_config().should_check_missing_streets(), "only");

    relations.activate_all(true);
    let active_names = relations.get_active_names().unwrap();
    assert_eq!(active_names.contains(&"myrelation2".to_string()), true);

    // Allow seeing data of a relation even if it's not in relations.yaml.
    relations.get_relation("gh195").unwrap();

    // Test limit_to_refcounty().
    // 01
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation1".to_string()),
        true
    );
    // 43
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation2".to_string()),
        true
    );
    relations
        .limit_to_refcounty(&Some("01".to_string()))
        .unwrap();
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation1".to_string()),
        true
    );
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation2".to_string()),
        false
    );

    // Test limit_to_refsettlement().
    // 011
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation1".to_string()),
        true
    );
    // 99
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation3".to_string()),
        true
    );
    relations.limit_to_refsettlement(&Some("99")).unwrap();
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation1".to_string()),
        false
    );
    assert_eq!(
        relations
            .get_active_names()
            .unwrap()
            .contains(&"myrelation3".to_string()),
        true
    );
}

/// Tests RelationConfig::should_check_missing_streets().
#[test]
fn test_relation_config_should_check_missing_streets() {
    let relation_name = "myrelation";
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            relation_name: {
                "refsettlement": "42",
            },
        },
        "relation-myrelation.yaml": {
            "missing-streets": "only",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation(relation_name).unwrap();
    let ret = relation.get_config().should_check_missing_streets();
    assert_eq!(ret, "only");
}

/// Tests RelationConfig::should_check_missing_streets(): the default.
#[test]
fn test_relation_config_should_check_missing_streets_default() {
    let relation_name = "myrelation";
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            relation_name: {
                "refsettlement": "42",
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation(relation_name).unwrap();
    let ret = relation.get_config().should_check_missing_streets();
    assert_eq!(ret, "yes");
}

/// Tests RelationConfig::get_letter_suffix_style().
#[test]
fn test_relation_config_get_letter_suffix_style() {
    let relation_name = "myrelation";
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            relation_name: {
                "refsettlement": "42",
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
    let mut relations = Relations::new(&ctx).unwrap();
    let mut relation = relations.get_relation(relation_name).unwrap();
    assert_eq!(
        relation.config.get_letter_suffix_style(),
        util::LetterSuffixStyle::Upper
    );
    let mut config = relation.config.clone();
    config.set_letter_suffix_style(util::LetterSuffixStyle::Lower);
    relation.set_config(&config);
    assert_eq!(
        relation.config.get_letter_suffix_style(),
        util::LetterSuffixStyle::Lower
    );
}

/// Tests refcounty_get_name().
#[test]
fn test_refcounty_get_name() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
        "refcounty-names.yaml": {
            "01": "Budapest",
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let relations = Relations::new(&ctx).unwrap();
    assert_eq!(relations.refcounty_get_name("01"), "Budapest");
    assert_eq!(relations.refcounty_get_name("99"), "");
}

/// Tests refcounty_get_refsettlement_ids().
#[test]
fn test_refcounty_get_refsettlement_ids() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
        "refcounty-names.yaml": {
            "01": "mycity",
        },
        "refsettlement-names.yaml": {
            "01": {
                "011": "myrelation1",
                "012": "myrelation1",
            }
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let relations = Relations::new(&ctx).unwrap();
    assert_eq!(
        relations.refcounty_get_refsettlement_ids("01"),
        ["011".to_string(), "012".to_string()]
    );
    assert_eq!(
        relations.refcounty_get_refsettlement_ids("99").is_empty(),
        true
    );
}

/// Tests refsettlement_get_name().
#[test]
fn test_refsettlement_get_name() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
        },
        "refcounty-names.yaml": {
            "01": "mycity",
        },
        "refsettlement-names.yaml": {
            "01": {
                "011": "mysettlement",
            }
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let relations = Relations::new(&ctx).unwrap();
    assert_eq!(
        relations.refsettlement_get_name("01", "011"),
        "mysettlement"
    );
    assert_eq!(relations.refsettlement_get_name("99", ""), "");
    assert_eq!(relations.refsettlement_get_name("01", "99"), "");
}

/// Tests Relalations::get_aliases().
#[test]
fn test_relations_get_aliases() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "budafok": {
            },
        },
        "relation-budafok.yaml": {
            "alias": ["budapest_22"],
        },
    });
    let yamls_cache_value = context::tests::TestFileSystem::write_json_to_file(&yamls_cache);
    let files = context::tests::TestFileSystem::make_files(
        &ctx,
        &[("data/yamls.cache", &yamls_cache_value)],
    );
    let file_system = context::tests::TestFileSystem::from_files(&files);
    ctx.set_file_system(&file_system);
    let mut relations = Relations::new(&ctx).unwrap();
    // Expect an alias -> canonicalname map.
    let mut expected = HashMap::new();
    expected.insert("budapest_22".to_string(), "budafok".to_string());
    assert_eq!(relations.get_aliases().unwrap(), expected);
}

/// Tests RelationConfig::get_street_is_even_odd().
#[test]
fn test_relation_config_get_street_is_even_odd() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "gazdagret": {
            },
        },
        "relation-gazdagret.yaml": {
            "filters": {
                "Hamzsabégi út": {
                    "interpolation": "all",
                },
                "Teszt utca": {
                    "interpolation": "notall",
                },
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("gazdagret").unwrap();
    assert_eq!(
        relation.config.get_street_is_even_odd("Hamzsabégi út"),
        false
    );

    assert_eq!(relation.config.get_street_is_even_odd("Teszt utca"), true);
}

/// Tests RelationConfig::should_show_ref_street().
#[test]
fn test_relation_config_should_show_ref_street() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
            },
        },
        "relation-myrelation.yaml": {
            "filters": {
                "mystreet1": {
                    "show-refstreet": false,
                },
                "mystreet2": {
                    "show-refstreet": true,
                },
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    assert_eq!(relation.config.should_show_ref_street("mystreet1"), false);
    assert_eq!(relation.config.should_show_ref_street("mystreet2"), true);
}

/// Tests RelationConfig::is_active().
#[test]
fn test_relation_config_is_active() {
    let relation_name = "myrelation";
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            relation_name: {
                "refsettlement": "42",
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation(relation_name).unwrap();
    assert_eq!(relation.get_config().is_active(), true);
}

/// Tests Relation::numbered_streets_to_table(): when a street is not even-odd.
#[test]
fn test_relation_numbered_streets_to_table() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
            },
        },
        "relation-myrelation.yaml": {
            "filters": {
                "mystreet": {
                    "interpolation": "all",
                }
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let street = util::Street::new("mystreet", "mystreet", false, 0);
    let house_numbers = vec![
        util::HouseNumber::new("1", "1", ""),
        util::HouseNumber::new("2", "2", ""),
    ];
    let streets = vec![(street, house_numbers)];

    let (table, _todo_count) = relation.numbered_streets_to_table(&streets);

    assert_eq!(table.len(), 2);
    // Ignore header.
    let row = &table[1];
    assert_eq!(row.len(), 3);
    assert_eq!(row[0].get_value(), "mystreet");
    assert_eq!(row[1].get_value(), "2");
    // No line break here.
    assert_eq!(row[2].get_value(), "1, 2");
}

/// Tests RelationConfig::set_active().
#[test]
fn test_relation_config_set_active() {
    let mut ctx = context::tests::make_test_context().unwrap();
    let yamls_cache = serde_json::json!({
        "relations.yaml": {
            "myrelation": {
                "osmrelation": 42,
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
    let mut relations = Relations::new(&ctx).unwrap();
    let relation = relations.get_relation("myrelation").unwrap();
    let mut config = relation.get_config().clone();
    assert_eq!(config.is_active(), true);
    config.set_active(false);
    assert_eq!(config.is_active(), false);
}
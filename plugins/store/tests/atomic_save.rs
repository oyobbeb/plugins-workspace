// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Saving truncated the store file and wrote the new contents into it, so a
// power loss or crash in between left an empty or zero-filled file (#3085).
// A save now writes a temporary file next to the store and renames it over.

use std::{fs, io::Read};

use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri_plugin_store::StoreBuilder;

fn store_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("store-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

// A reader that opened the file before a save keeps reading the old file whole:
// the save replaced the file instead of rewriting it in place. On Unix an open
// file outlives a rename over its name, which is what makes this observable.
#[cfg(unix)]
#[test]
fn saving_replaces_the_store_file_instead_of_rewriting_it() {
    let app = mock_builder()
        .plugin(tauri_plugin_store::Builder::new().build())
        .build(mock_context(noop_assets()))
        .unwrap();
    let path = store_dir("replace").join("settings.json");

    let store = StoreBuilder::new(&app, &path)
        .disable_auto_save()
        .build()
        .unwrap();
    store.set("theme", "dark");
    store.save().unwrap();
    let before = fs::read_to_string(&path).unwrap();
    let mut reader = fs::File::open(&path).unwrap();

    store.set("theme", "light");
    store.save().unwrap();

    let mut seen = String::new();
    reader.read_to_string(&mut seen).unwrap();
    assert_eq!(seen, before);
    assert!(fs::read_to_string(&path).unwrap().contains("light"));
}

// Runs on every platform, so Windows also renames over an existing file here.
#[test]
fn saving_leaves_only_the_store_file() {
    let app = mock_builder()
        .plugin(tauri_plugin_store::Builder::new().build())
        .build(mock_context(noop_assets()))
        .unwrap();
    let dir = store_dir("leftovers");
    let path = dir.join("settings.json");

    let store = StoreBuilder::new(&app, &path)
        .disable_auto_save()
        .build()
        .unwrap();
    for i in 0..3 {
        store.set("count", i);
        store.save().unwrap();
    }

    let names: Vec<_> = fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, ["settings.json"]);
    assert!(fs::read_to_string(&path).unwrap().contains('2'));
}

//! Собирает `libsmart_socket_ffi.a` и подключает ее к приложению.
//!
//! Cargo не умеет передавать зависимому пакету артефакт `staticlib`,
//! поэтому библиотека собирается вложенным вызовом cargo в отдельный
//! каталог внутри `OUT_DIR` — так он не конкурирует с внешней сборкой
//! за блокировку `target`.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("задается cargo"));
    let ffi_dir = manifest_dir.join("../smart-socket-ffi");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("задается cargo"));
    let target_dir = out_dir.join("ffi");
    let target = env::var("TARGET").expect("задается cargo");
    let profile = env::var("PROFILE").expect("задается cargo");

    let mut cargo = Command::new(env::var("CARGO").expect("задается cargo"));
    cargo
        .arg("build")
        .arg("--lib")
        .arg("--manifest-path")
        .arg(ffi_dir.join("Cargo.toml"))
        .arg("--target-dir")
        .arg(&target_dir)
        .arg("--target")
        .arg(&target)
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .env_remove("RUSTC_WRAPPER");
    if profile == "release" {
        cargo.arg("--release");
    }
    let status = cargo.status().expect("cargo запускается");
    assert!(status.success(), "не удалось собрать smart-socket-ffi");

    let lib_name = if target.contains("windows-msvc") {
        "smart_socket_ffi.lib"
    } else {
        "libsmart_socket_ffi.a"
    };
    let lib_dir = out_dir.join("lib");
    std::fs::create_dir_all(&lib_dir).expect("каталог создается");
    std::fs::copy(
        target_dir.join(&target).join(&profile).join(lib_name),
        lib_dir.join(lib_name),
    )
    .expect("статическая библиотека собрана");
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rerun-if-changed={}", ffi_dir.join("src").display());
    println!(
        "cargo:rerun-if-changed={}",
        ffi_dir.join("Cargo.toml").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("../../src").display()
    );
}

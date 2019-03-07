/* Copyright (c) Fortanix, Inc.
 *
 * Licensed under the GNU General Public License, version 2 <LICENSE-GPL or
 * https://www.gnu.org/licenses/gpl-2.0.html> or the Apache License, Version
 * 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>, at your
 * option. This file may not be copied, modified, or distributed except
 * according to those terms. */

extern crate bindgen;
extern crate cmake;

mod config;
mod headers;
#[path = "bindgen.rs"]
mod mod_bindgen;
#[path = "cmake.rs"]
mod mod_cmake;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::io::prelude::*;
use std::process::Command;
use std::path::{Path, PathBuf};

pub fn have_feature(feature: &'static str) -> bool {
    env::var_os(
        format!("CARGO_FEATURE_{}", feature)
            .to_uppercase()
            .replace("-", "_"),
    )
    .is_some()
}

struct BuildConfig {
    out_dir: PathBuf,
    mbedtls_src: PathBuf,
    config_h: PathBuf,
}

impl BuildConfig {
    fn create_config_h(&self) {
        let target = env::var("TARGET").unwrap();
        let mut defines = config::DEFAULT_DEFINES
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>();
        for &(feat, def) in config::FEATURE_DEFINES {
            if (feat == "std") && (target == "x86_64-fortanix-unknown-sgx") {
                continue;
            }
            if have_feature(feat) {
                defines.insert(def.0, def.1);
            }
        }

        File::create(&self.config_h)
            .and_then(|mut f| {
                try!(f.write_all(config::PREFIX.as_bytes()));
                for (name, def) in defines {
                    try!(f.write_all(def.define(name).as_bytes()));
                }
                if have_feature("custom_printf") {
                    try!(writeln!(f, "int mbedtls_printf(const char *format, ...);"));
                }
                if have_feature("custom_threading") {
                    try!(writeln!(f, "typedef void* mbedtls_threading_mutex_t;"));
                }
                f.write_all(config::SUFFIX.as_bytes())
            })
            .expect("config.h I/O error");
    }

    fn print_rerun_files(&self) {
        println!("cargo:rerun-if-env-changed=RUST_MBEDTLS_SYS_SOURCE");
        println!(
            "cargo:rerun-if-changed={}",
            self.mbedtls_src.join("CMakeLists.txt").display()
        );
        let include = self.mbedtls_src.join(Path::new("include").join("mbedtls"));
        for h in headers::enabled_ordered() {
            println!("cargo:rerun-if-changed={}", include.join(h).display());
        }
        for f in self
            .mbedtls_src
            .join("library")
            .read_dir()
            .expect("read_dir failed")
        {
            println!(
                "cargo:rerun-if-changed={}",
                f.expect("DirEntry failed").path().display()
            );
        }
    }
}

fn run_conan() {
    let profile = env::var("PROFILE").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let build_type = if profile == "debug" { "Debug" } else { "Release" };
    let mut conan_file = manifest_dir.to_path_buf();
    conan_file.push("build");
    conan_file.push("conanfile.txt");

    Command::new("conan")
        .arg("install")
        .arg("-pr")
        .arg(format!("{}-{}", &target_os, &target_arch))
        .arg("-s")
        .arg(format!("build_type={}", &build_type))
        .arg("-if")
        .arg(&out_dir)
        .arg(&conan_file.to_str().unwrap())
        .output()
        .expect("failed to execute conan");

    let mut conan_build_info = out_dir.clone();
    conan_build_info.push("conanbuildinfo.cargo");

    let mut file = File::open(conan_build_info).expect("Error opening conanbuildinfo.cargo");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read to string");
    println!("{}", contents);
}

fn main() {
    if let Ok(_) = env::var("CARGO_FEATURE_CONAN_BUILD") {
        run_conan();
        return;
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR environment not set?"));
    let src = PathBuf::from(env::var("RUST_MBEDTLS_SYS_SOURCE").unwrap_or("vendor".to_owned()));
    let cfg = BuildConfig {
        config_h: out_dir.join("config.h"),
        out_dir: out_dir,
        mbedtls_src: src,
    };

    cfg.create_config_h();
    cfg.print_rerun_files();
    cfg.cmake();
    cfg.bindgen();
}

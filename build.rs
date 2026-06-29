use cxx_build::CFG;
use std::fs::canonicalize;
use std::path::PathBuf;

fn main() {
    let mut cmake_config = cmake::Config::new("spoa");
    cmake_config
        .define("spoa_install", "OFF")
        .define("spoa_build_exe", "OFF")
        .define("spoa_build_tests", "OFF");

    // Disable -march=native for portable builds
    cmake_config.define("spoa_optimize_for_native", "OFF");
    cmake_config.define("spoa_use_simde", "ON");

    // spoa_generate_dispatch compiles x86-specific dispatch objects (-mavx2, -msse4.1,
    // -msse2) that are incompatible with ARM targets; only enable on x86.
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let is_x86 = target_arch == "x86" || target_arch == "x86_64";
    cmake_config.define("spoa_generate_dispatch", if is_x86 { "ON" } else { "OFF" });

    // CMake >= 4.0 removed compatibility with cmake_minimum_required < 3.5;
    // cpu_features v0.6.0 declares VERSION 3.0, so we must override the policy
    // floor here until cpu_features is updated upstream.
    cmake_config.define("CMAKE_POLICY_VERSION_MINIMUM", "3.5");

    let out_dir = cmake_config.build();

    println!(
        "cargo:rustc-link-search=native={}/build/lib",
        out_dir.display()
    );
    println!("cargo:rustc-link-lib=spoa");
    // cpu_features is only built when generate_dispatch is ON (x86 only)
    if is_x86 {
        println!("cargo:rustc-link-lib=static=cpu_features");
    }

    let spoa_include = canonicalize(PathBuf::from("spoa/include")).unwrap();
    CFG.exported_header_dirs.push(&spoa_include);

    cxx_build::bridge("src/lib.rs")
        .cpp(true)
        .file("cxx/spoa_rs.cpp")
        .flag_if_supported("-std=c++14")
        .compile("spoa_rs");

    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=cxx/spoa_rs.hpp");
    println!("cargo:rerun-if-changed=cxx/spoa_rs.cpp");
    println!("cargo:rerun-if-changed=spoa/spoa.hpp");
}

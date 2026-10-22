// 开发/构建统一使用官方预编译 libduckdb（动态链接），负责：
// 1. 校验 libduckdb 存在（.cargo/config.toml 的 [env] 或外部环境变量提供路径），
//    缺失时给出可执行的修复指引（跑 scripts/dev/setup-duckdb.ps1|.sh）
// 2. 拦截 duckdb-bundled 自编译 feature（自编版在 Windows 上 C++ 异常穿透 FFI
//    会 abort 整个进程，见 DEPLOY.md / tests/duckdb_error_paths.rs）
// 3. Windows：复制 duckdb.dll 到 profile 目录与 deps/（cargo run / cargo test
//    的 exe 都能找到 DLL）
// 4. Linux：嵌入 rpath，运行时自动定位 vendor 目录里的 libduckdb.so
use std::{
    env,
    path::{Path, PathBuf},
};

const DUCKDB_VERSION: &str = "1.5.3"; // 与 Cargo.toml 的 duckdb crate 同步（1.10503.x ↔ DuckDB 1.5.3）

fn main() {
    // bundled 自编译一律拦截
    if env::var_os("CARGO_FEATURE_DUCKDB_BUNDLED").is_some() {
        panic!(
            "已禁用 duckdb-bundled 自编译（自编 libduckdb 有致命缺陷：Windows 上\n\
             DuckDB 错误会以 C++ 异常穿透 FFI abort 进程）。\n\
             请去掉 --features duckdb-bundled，默认的官方预编译动态链接即可。"
        );
    }

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let lib_dir = resolve_lib_dir();

    // 校验平台关键文件
    let (lib_file, dll_file) = if target_os == "windows" {
        ("duckdb.lib", Some("duckdb.dll"))
    } else {
        ("libduckdb.so", Some("libduckdb.so"))
    };
    let lib_path = lib_dir.join(lib_file);
    if !lib_path.exists() {
        panic!(
            "未找到官方预编译 libduckdb（{}）。\n\
             期望目录: {}\n\
             请先执行: scripts/dev/setup-duckdb{} （当前 DUCKDB_LIB_DIR={}）\n\
             版本需与 Cargo.toml 的 duckdb crate 匹配：DuckDB {}",
            lib_path.display(),
            lib_dir.display(),
            if target_os == "windows" {
                ".ps1"
            } else {
                ".sh"
            },
            lib_dir.display(),
            DUCKDB_VERSION,
        );
    }
    if !lib_dir.join("duckdb.h").exists() {
        panic!("libduckdb 目录缺 duckdb.h: {}", lib_dir.display());
    }

    println!("cargo:rerun-if-env-changed=DUCKDB_LIB_DIR");
    println!("cargo:rerun-if-changed={}", lib_path.display());

    if target_os == "windows" {
        deploy_windows_dll(&lib_dir, dll_file.unwrap());
    } else {
        // rpath 让 cargo run / cargo test / 直接执行 exe 都能找到 libduckdb.so
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            lib_dir.to_string_lossy()
        );
    }
}

/// 解析 libduckdb 目录：优先 DUCKDB_LIB_DIR（.cargo/config.toml [env] 或外部
/// 环境变量），兜底仓库约定位置（相对本包根：../scripts/build/vendor/libduckdb-current）。
fn resolve_lib_dir() -> PathBuf {
    if let Some(dir) = env::var_os("DUCKDB_LIB_DIR") {
        let p = PathBuf::from(&dir);
        if p.is_absolute() {
            return p;
        }
        // 相对路径：libduckdb-sys 的 build.rs 以自身包根为 CWD，无法解析仓库相对路径，
        // 这里基于本包根兜底解析（本包根 = hdata-analysis-rs，../scripts 即仓库 scripts）
        if let Some(manifest) = env::var_os("CARGO_MANIFEST_DIR") {
            let abs = Path::new(&manifest).join(&dir);
            if abs.exists() {
                // 同时把绝对路径回写给后续编译产物（含 libduckdb-sys 需要时）
                println!("cargo:rustc-env=DUCKDB_LIB_DIR={}", abs.display());
                return abs;
            }
        }
        return p;
    }
    // 兜底：约定目录（config.toml 未生效时，例如从仓库根以外执行 cargo）
    let manifest = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("CARGO_MANIFEST_DIR 必定存在");
    manifest
        .join("../scripts/build/vendor/libduckdb-current")
        .canonicalize()
        .unwrap_or_else(|_| {
            manifest.join("../scripts/build/vendor/libduckdb-current")
        })
}

/// Windows：复制 duckdb.dll 到 target/<profile>/ 与 target/<profile>/deps/，
/// cargo run（exe 在 profile 根）与 cargo test（exe 在 deps/）都能加载。
/// exe 正在运行时 DLL 被锁定，复制失败仅告警不阻断构建。
fn deploy_windows_dll(lib_dir: &Path, dll: &str) {
    let Some(out_dir) = env::var_os("OUT_DIR").map(PathBuf::from) else {
        return;
    };
    // OUT_DIR = target/<profile>/build/<pkg>-<hash>/out → 上三级为 <profile>
    let Some(profile_dir) = out_dir.ancestors().nth(3) else {
        return;
    };
    let src = lib_dir.join(dll);
    for dest in [profile_dir.to_path_buf(), profile_dir.join("deps")] {
        let target = dest.join(dll);
        if let Err(e) = std::fs::copy(&src, &target) {
            // 目标 DLL 被运行中的进程锁定等场景：提示但不失败
            println!(
                "cargo:warning=复制 {} 失败（若程序正在运行可忽略）: {}",
                target.display(),
                e
            );
        }
    }
}

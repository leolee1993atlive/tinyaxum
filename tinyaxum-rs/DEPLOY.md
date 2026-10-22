# 部署清单（hdata-analysis-rs / Windows）

本服务在**开发机**用 `cargo build --release` 编译，把产物拷到**生产机**（Windows Server 2022，
用 NSSM/WinSW 注册为服务）"直接部署"。生产机**没有 Rust 工具链、可能无外网**。

---

## 一、构建：链接官方预编译 libduckdb（不要用 `bundled` 自编）

> **为什么不用 `bundled`**：`duckdb` 的 `bundled` feature 会用 cc 从源码自编 DuckDB，
> 产出的二进制在生产机一初始化就崩（`duckdb_create_config` panic / 被吞成 "Unknown error"），
> 各版本（1.5.2 / 1.5.3 / 1.4.4）都崩；而同机器上官方预编译的 DuckDB（CLI / dll）正常。
> 所以改成链接官方预编译库。

1. 从 <https://duckdb.org> 下载 **C/C++** 包 `libduckdb-windows-amd64.zip`，版本 **v1.5.3**，
   解压后含 `duckdb.dll` / `duckdb.lib` / `duckdb.h`。
2. 设环境变量后构建：
   ```bat
   set DUCKDB_LIB_DIR=<解压目录>
   set DUCKDB_INCLUDE_DIR=<解压目录>
   cargo build --release
   ```
   `Cargo.toml` 里 `duckdb` 按平台分:`[target.'cfg(windows)'.dependencies]` 用
   `["r2d2"]`（**不带 `bundled`**），编译期才会去链接 `DUCKDB_LIB_DIR` 里的官方库。
   （Linux/Docker 走另一支 `cfg(not(windows))`，仍是 `bundled`，见第六节。）

---

## 二、部署：这些文件必须和 exe 放在**同一目录**

```
<部署目录>\
  hdata-analysis.exe
  duckdb.dll                          ← 官方 libduckdb（v1.5.3/windows_amd64），动态链接，必带
  config.toml
  extensions\
    mysql.duckdb_extension            ← MySQL 扩展（离线加载）
    postgres.duckdb_extension         ← 用到 PostgreSQL 才需要
```

- **`duckdb.dll` 漏带** → 启动报"找不到 duckdb.dll"/加载失败。
- **`extensions\` 漏带** → 同步时报加载扩展失败（生产机无外网无法在线 `INSTALL`）。
- 代码逻辑在 `src/util/duckdb_util.rs`：exe 同目录有 `extensions\<ext>.duckdb_extension`
  就离线 `LOAD`，否则联网 `INSTALL`（仅供开发机用）。

---

## 三、版本耦合（升级 DuckDB 时三者必须一起换）

`duckdb` crate 版本、`duckdb.dll`、`extensions\*.duckdb_extension`
**必须同为同一个 DuckDB 版本**（当前 **1.5.3**，平台 **windows_amd64**）。任一不一致就会加载/运行报错。

---

## 四、离线扩展文件怎么拿

- 有网机器下载（注意是 `.gz`，解压去掉后缀）：
  ```
  https://extensions.duckdb.org/v1.5.3/windows_amd64/mysql.duckdb_extension.gz
  https://extensions.duckdb.org/v1.5.3/windows_amd64/postgres.duckdb_extension.gz
  ```
- 或从用过官方 CLI（**版本须为 1.5.3**）的机器缓存里拷：
  ```
  %USERPROFILE%\.duckdb\extensions\v1.5.3\windows_amd64\mysql.duckdb_extension
  ```

---

## 五、诊断工具 `duckdb_diag`（保留，勿删）

`src/bin/duckdb_diag.rs` 是常驻诊断程序，用来在任意机器上验证 DuckDB 能否初始化、
并打印出 `duckdb_create_config` 平时吞掉的真实异常：

```bat
cargo build --release --bin duckdb_diag
```

把 `duckdb_diag.exe` + `duckdb.dll` 放同目录运行：
- `结果: OK` → DuckDB 初始化正常；
- `结果: 失败 … 真实异常 = …` → 打印底层原因。

---

## 六、Linux / Docker（对照，和上面的 Windows 清单不同）

以上一至五节**只针对 Windows**。Linux/Docker **不走外部库**：`Cargo.toml` 的
`[target.'cfg(not(windows))'.dependencies]` 仍用 `bundled`，DuckDB 静态编进二进制。所以：

- **不需要** `libduckdb.so` / `duckdb.dll`，部署就一个可执行文件（+ `config.toml`），最简单；
- 构建照旧 `scripts/build/build-linux.sh` / `build-docker.sh`，**无需**设 `DUCKDB_LIB_DIR`/`DUCKDB_INCLUDE_DIR`；
- 扩展（mysql/postgres）仍是运行时加载：默认联网 `INSTALL`；若 Linux 生产机也无外网，
  同样把 **`linux_amd64`** 平台的 `*.duckdb_extension` 放进二进制同目录的 `extensions/`
  即可离线 `LOAD`（机制与 Windows 相同，只是平台串换成 `linux_amd64`）。

> 为什么 Windows 特殊:cc 自编的 DuckDB 在生产 Windows 初始化即崩(见第一节),Linux 的
> bundled 没这问题且更省事,故只有 Windows 改用外部官方库。

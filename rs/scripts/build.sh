#!/bin/sh


# 统一构建（单项目：在仓库根目录直接构建）
cd .. || exit 1
cargo build --release || exit 1

# 使用 cbindgen 生成头文件（如已安装）。
if command -v cbindgen >/dev/null 2>&1; then
  echo "Generating C header via cbindgen..."
  cbindgen --config cbindgen.toml --crate sqler --output include/sqler.h || exit 1
else
  echo "cbindgen not found; skipping header generation."
fi

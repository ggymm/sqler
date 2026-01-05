SHELL := bash
.SHELLFLAGS := -eu -o pipefail -c

UNAME_S := $(shell uname -s 2>/dev/null)
UNAME_M := $(shell uname -m 2>/dev/null)

# 检测操作系统
ifeq ($(OS),Windows_NT)
  HOST_OS := windows
else ifeq ($(UNAME_S),Darwin)
  HOST_OS := macos
else ifeq ($(UNAME_S),Linux)
  HOST_OS := linux
else
  HOST_OS := unknown
endif

# 检测架构并规范化
ifeq ($(UNAME_M),x86_64)
  HOST_ARCH := x86_64
else ifeq ($(UNAME_M),aarch64)
  HOST_ARCH := aarch64
else ifeq ($(UNAME_M),arm64)
  HOST_ARCH := aarch64
else
  HOST_ARCH := unknown
endif

CARGO ?= cargo
PACKAGE_MAIN ?= sqler-app
PACKAGE_TASK ?= sqler-task

# 根据当前平台生成 target triple
ifeq ($(HOST_OS),linux)
  ifeq ($(HOST_ARCH),x86_64)
    TARGET := x86_64-unknown-linux-gnu
  else ifeq ($(HOST_ARCH),aarch64)
    TARGET := aarch64-unknown-linux-gnu
  else
    TARGET := unknown
  endif
else ifeq ($(HOST_OS),macos)
  ifeq ($(HOST_ARCH),x86_64)
    TARGET := x86_64-apple-darwin
  else ifeq ($(HOST_ARCH),aarch64)
    TARGET := aarch64-apple-darwin
  else
    TARGET := unknown
  endif
else ifeq ($(HOST_OS),windows)
  ifeq ($(HOST_ARCH),x86_64)
    TARGET := x86_64-pc-windows-msvc
  else ifeq ($(HOST_ARCH),aarch64)
    TARGET := aarch64-pc-windows-msvc
  else
    TARGET := unknown
  endif
else
  TARGET := unknown
endif

# Windows FXC 路径设置（仅 Windows 构建使用）
# 通过注册表自动发现 Windows SDK 安装路径
define setup_windows_fxc
	if [ -z "$${GPUI_FXC_PATH:-}" ]; then \
		export GPUI_FXC_PATH=$$(reg.exe query "HKLM\\SOFTWARE\\Microsoft\\Windows Kits\\Installed Roots" //v KitsRoot10 2>/dev/null \
			| grep "REG_SZ" | sed 's/.*REG_SZ[[:space:]]*//' | tr -d '\r' \
			| sed 's#\\#/#g' | sed 's#^\([A-Z]\):#/\L\1#' | sed 's#/$$##' \
			| while read -r sdk_path; do \
				for ver_dir in "$$sdk_path/bin"/*/ ; do \
					[ -f "$${ver_dir}x64/fxc.exe" ] && echo "$${ver_dir}x64/fxc.exe"; \
				done; \
			done | sort | tail -1); \
		[ -z "$$GPUI_FXC_PATH" ] && { \
			echo "错误: 未找到 Windows SDK 或 fxc.exe" >&2; \
			echo "请安装 Windows 10 SDK 或设置环境变量 GPUI_FXC_PATH" >&2; \
			exit 1; \
		}; \
	fi;
endef

.PHONY: help fmt clean update package \
	bloat-top bloat-crates outdated ensure-cargo-bloat ensure-cargo-outdated

help:
	@printf "Sqler Makefile 可用命令\n"
	@printf "\n当前主机环境: HOST_OS=$(HOST_OS), HOST_ARCH=$(HOST_ARCH)\n"
	@printf "构建目标: TARGET=$(TARGET)\n\n"
	@printf "  %-32s %s\n" "make help" "显示本帮助信息"
	@printf "  %-32s %s\n" "make clean" "运行 cargo clean 清理构建产物"
	@printf "  %-32s %s\n" "make format" "运行 cargo fmt --all 格式化代码"
	@printf "  %-32s %s\n" "make update" "运行 cargo update 升级依赖锁文件"
	@printf "  %-32s %s\n" "make package" "为当前平台构建 ($(HOST_OS)/$(HOST_ARCH))"
	@printf "  %-32s %s\n" "make bloat-top" "分析二进制体积（前 50 个函数）"
	@printf "  %-32s %s\n" "make bloat-crates" "按 crate 汇总二进制体积"
	@printf "  %-32s %s\n" "make outdated" "检查过时的依赖项"

clean:
	$(CARGO) clean

format:
	$(CARGO) fmt --all

update:
	$(CARGO) update

package:
	@if [ "$(TARGET)" = "unknown" ]; then \
		echo "错误: 不支持的平台: $(HOST_OS)/$(HOST_ARCH)" >&2; \
		exit 1; \
	fi
	@printf "==> 构建 $(TARGET)\n"
	@if [ "$(HOST_OS)" = "macos" ]; then \
		ulimit -n 10240; \
	fi
ifeq ($(HOST_OS),windows)
	@$(call setup_windows_fxc) \
	printf "==> 构建主应用: $(PACKAGE_MAIN)\n"; \
	$(CARGO) build --locked --release --target $(TARGET) --package $(PACKAGE_MAIN); \
	printf "==> 构建任务执行器: $(PACKAGE_TASK)\n"; \
	$(CARGO) build --locked --release --target $(TARGET) --package $(PACKAGE_TASK); \
	if command -v upx >/dev/null 2>&1; then \
		printf "==> 使用 UPX 压缩二进制文件\n"; \
		upx --best --lzma target/$(TARGET)/release/sqler.exe 2>/dev/null || true; \
		upx --best --lzma target/$(TARGET)/release/sqler-task.exe 2>/dev/null || true; \
		printf "==> UPX 压缩完成\n"; \
	else \
		printf "警告: 未找到 UPX，跳过压缩步骤\n"; \
		printf "提示: 可通过 'scoop install upx' 或 'choco install upx' 安装 UPX\n"; \
	fi
else
	@printf "==> 构建主应用: $(PACKAGE_MAIN)\n"
	@$(CARGO) build --locked --release --target $(TARGET) --package $(PACKAGE_MAIN)
	@printf "==> 构建任务执行器: $(PACKAGE_TASK)\n"
	@$(CARGO) build --locked --release --target $(TARGET) --package $(PACKAGE_TASK)
endif

ensure-cargo-bloat:
	@if ! command -v cargo-bloat >/dev/null 2>&1; then \
		echo "未找到 cargo-bloat，正在安装..."; \
		$(CARGO) install cargo-bloat; \
	fi

bloat-top: ensure-cargo-bloat
	$(CARGO) bloat --release -n 50

bloat-crates: ensure-cargo-bloat
	$(CARGO) bloat --release --crates

ensure-cargo-outdated:
	@if ! command -v cargo-outdated >/dev/null 2>&1; then \
		echo "未找到 cargo-outdated，正在安装..."; \
		$(CARGO) install cargo-outdated; \
	fi

outdated: ensure-cargo-outdated
	$(CARGO) outdated -R

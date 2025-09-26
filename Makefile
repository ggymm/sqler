.DEFAULT_GOAL := help

UNAME_S := $(shell uname -s)

.PHONY: help build-debug build-release run-debug run-release deploy-debug deploy-release clean \
	format-qml lint-qml format-cpp format-all lint-cpp tidy-fix-cpp

help:
	@echo "commands:"
	@echo "  make run-debug      - Run Debug build"
	@echo "  make run-release    - Run Release build"
	@echo "  make build-debug    - Configure + build Debug (build/Debug)"
	@echo "  make build-release  - Configure + build Release (build/Release)"
	@echo "  make deploy-debug   - Deploy Debug build with Qt runtime"
	@echo "  make deploy-release - Deploy Release build with Qt runtime"
	@echo "  make clean          - Remove build directory"
	@echo "  make lint-qml       - Lint all QML via qmllint"
	@echo "  make format-qml     - Format all QML via qmlformat"
	@echo "  make format-cpp     - Format all C/C++ via clang-format"
	@echo "  make lint-cpp       - Lint C/C++ via clang-tidy (if available)"
	@echo "  make tidy-fix-cpp   - Auto-fix C/C++ via clang-tidy (if available)"
	@echo "  make format-all     - Format all QML and C/C++"


run-debug: build-debug
ifeq ($(UNAME_S),Darwin)
	open build/Debug/sqler.app || build/Debug/sqler.app/Contents/MacOS/sqler || true
else
	build/Debug/sqler
endif

run-release: build-release
ifeq ($(UNAME_S),Darwin)
	open build/Release/sqler.app || build/Release/sqler.app/Contents/MacOS/sqler || true
else
	build/Release/sqler
endif

build-debug:
	cmake -S . -B build/Debug -DCMAKE_BUILD_TYPE=Debug
	cmake --build build/Debug -j

build-release:
	cmake -S . -B build/Release -DCMAKE_BUILD_TYPE=Release
	cmake --build build/Release -j

deploy-debug: build-debug
	cmake --build build/Debug -t deploy

deploy-release: build-release
	cmake --build build/Release -t deploy

clean:
	rm -rf build

lint-qml:
	@echo "Linting QML files..."
	@rg -l --glob "*.qml" assets/qml | while read -r f; do qmllint "$$f"; done

format-qml:
	@echo "Formatting QML files..."
	@rg -l --glob "*.qml" assets/qml | while read -r f; do qmlformat -i "$$f"; done

format-cpp:
	@echo "Formatting C/C++ files..."
	@if command -v clang-format >/dev/null 2>&1; then \
		find src -type f \( -name "*.c" -o -name "*.cc" -o -name "*.cpp" -o -name "*.cxx" -o -name "*.h" -o -name "*.hh" -o -name "*.hpp" -o -name "*.hxx" \) -exec clang-format -i {} + ; \
	else \
		echo "clang-format not found; install it to format C/C++"; \
	fi

format-all: format-qml format-cpp

lint-cpp:
	@echo "Linting C/C++ with clang-tidy..."
	@if ! command -v clang-tidy >/dev/null 2>&1; then \
		echo "clang-tidy not found; install it to enable C/C++ lint"; \
		exit 0; \
	fi
	@DB=""; \
	if [ -f build/Debug/compile_commands.json ]; then DB=build/Debug; \
	elif [ -f build/Release/compile_commands.json ]; then DB=build/Release; \
	fi; \
	FILES=$$(find src -type f \( -name "*.c" -o -name "*.cc" -o -name "*.cpp" -o -name "*.cxx" \)); \
	if [ -z "$$FILES" ]; then echo "No C/C++ sources under src"; exit 0; fi; \
	for f in $$FILES; do \
		if [ -n "$$DB" ]; then \
			clang-tidy -p "$$DB" "$${f}" --warnings-as-errors='*' || exit $$?; \
		else \
			clang-tidy "$${f}" --warnings-as-errors='*' || exit $$?; \
		fi; \
	done

tidy-fix-cpp:
	@echo "Applying clang-tidy auto-fixes..."
	@if ! command -v clang-tidy >/dev/null 2>&1; then \
		echo "clang-tidy not found; install it to enable auto-fix"; \
		exit 0; \
	fi
	@DB=""; \
	if [ -f build/Debug/compile_commands.json ]; then DB=build/Debug; \
	elif [ -f build/Release/compile_commands.json ]; then DB=build/Release; \
	fi; \
	FILES=$$(find src -type f \( -name "*.c" -o -name "*.cc" -o -name "*.cpp" -o -name "*.cxx" \)); \
	if [ -z "$$FILES" ]; then echo "No C/C++ sources under src"; exit 0; fi; \
	for f in $$FILES; do \
		if [ -n "$$DB" ]; then \
			clang-tidy -p "$$DB" "$${f}" --fix --format-style=file --warnings-as-errors='*' || exit $$?; \
		else \
			clang-tidy "$${f}" --fix --format-style=file --warnings-as-errors='*' || exit $$?; \
		fi; \
	done

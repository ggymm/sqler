.DEFAULT_GOAL := help

UNAME_S := $(shell uname -s)

help:
	@echo "commands:"
	@echo "  make run-debug      - Run Debug build"
	@echo "  make run-release    - Run Release build"
	@echo "  make build-debug    - Configure + build Debug (build/Debug)"
	@echo "  make build-release  - Configure + build Release (build/Release)"
	@echo "  make format         - Format all .cpp and .h files with clang-format"
	@echo "  make clean          - Remove build directory"


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
	@cmake -S . -B build/Debug -DCMAKE_BUILD_TYPE=Debug
	@cmake --build build/Debug -j

build-release:
	@cmake -S . -B build/Release -DCMAKE_BUILD_TYPE=Release
	@cmake --build build/Release -j

clean:
	@rm -rf build

format:
	@find . -name "*.cpp" -o -name "*.h" | xargs clang-format -i

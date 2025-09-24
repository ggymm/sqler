.DEFAULT_GOAL := help

UNAME_S := $(shell uname -s)

.PHONY: help build-debug build-release run-debug run-release deploy-debug deploy-release clean

help:
	@echo "Available commands:"
	@echo "  make run-debug      - Run Debug build"
	@echo "  make run-release    - Run Release build"
	@echo "  make build-debug    - Configure + build Debug (build/Debug)"
	@echo "  make build-release  - Configure + build Release (build/Release)"
	@echo "  make deploy-debug   - Deploy Debug build with Qt runtime"
	@echo "  make deploy-release - Deploy Release build with Qt runtime"
	@echo "  make clean          - Remove build directory"

run-debug: build-debug
ifeq ($(UNAME_S),Darwin)
	open build/Debug/SimpleQtApp.app
else
	build/Debug/SimpleQtApp
endif

run-release: build-release
ifeq ($(UNAME_S),Darwin)
	open build/Release/SimpleQtApp.app
else
	build/Release/SimpleQtApp
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

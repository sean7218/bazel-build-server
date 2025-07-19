# Makefile for Bazel SourceKit BSP

EXECUTABLE_NAME = bazel-build-server
INSTALL_DIR = /usr/local/bin

.PHONY: all build release run clean install

all: build

build:
	swift build

release:
	swift build --configuration release

run: build
	swift run $(EXECUTABLE_NAME)

clean:
	swift package clean

install: release
	sudo cp .build/arm64-apple-macosx/release/$(EXECUTABLE_NAME) $(INSTALL_DIR)/ 

test-harness:
	swift build --configuration debug
	cp .build/arm64-apple-macosx/debug/$(EXECUTABLE_NAME) TestHarness/
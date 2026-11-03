ROOT := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
export CARGO_TARGET_DIR := $(ROOT)/target

APP_CRATE := starter-app
BIN := rs-starter-template
CARGO ?= cargo
RUN_ARGS ?=

.DEFAULT_GOAL := help

.PHONY: help build release run run-debug test fmt fmt-check lint coverage check \
        clean distclean

help:
	@echo "Rust Starter Template — build targets"
	@echo
	@echo "Rust:"
	@echo "  build               debug-build the workspace"
	@echo "  release             build the release binary"
	@echo "  run / run-debug     run the release / debug binary"
	@echo "  test                run workspace tests"
	@echo "  fmt / fmt-check     format / check Rust code"
	@echo "  lint                run clippy with warnings denied"
	@echo "  coverage            generate coverage and clippy reports"
	@echo "  check               run all Rust checks"
	@echo
	@echo "Other:"
	@echo "  clean               clean Rust build artifacts"
	@echo "  distclean           clean all local build artifacts"

build:
	$(CARGO) build --workspace --locked

release:
	$(CARGO) build --release -p $(APP_CRATE) --locked
	@ls -lh $(CARGO_TARGET_DIR)/release/$(BIN)

run:
	$(CARGO) run --release -p $(APP_CRATE) --locked -- $(RUN_ARGS)

run-debug:
	$(CARGO) run -p $(APP_CRATE) --locked -- $(RUN_ARGS)

test:
	$(CARGO) test --workspace --locked

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

lint:
	$(CARGO) clippy --workspace --all-targets --all-features --locked -- -D warnings

coverage:
	mkdir -p coverage
	$(CARGO) llvm-cov --workspace --locked --lcov --output-path coverage/lcov.info
	$(CARGO) clippy --workspace --all-targets --all-features --locked --message-format=json > coverage/clippy.json

check: fmt-check lint test

clean:
	$(CARGO) clean

distclean: clean
	rm -rf coverage

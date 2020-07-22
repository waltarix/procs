ifeq ($(RUST_TARGET),)
	TARGET := ""
	RELEASE_SUFFIX := ""
else
	TARGET := $(RUST_TARGET)
	RELEASE_SUFFIX := "-$(TARGET)"
	export CARGO_BUILD_TARGET = $(RUST_TARGET)
endif

VERSION := $(word 3,$(shell grep -m1 "^version" Cargo.toml))
RELEASE := procs-$(VERSION)$(RELEASE_SUFFIX)

all: release

procs:
	cargo build --locked --release

bin:
	mkdir -p $@

bin/procs: procs bin
	cp -f target/$(TARGET)/release/procs $@

release: bin/procs
	tar -C bin -Jcvf $(RELEASE).tar.xz procs

.PHONY: all release

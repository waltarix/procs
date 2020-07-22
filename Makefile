OSTYPE := $(shell uname -s)
ifeq ($(OSTYPE),Linux)
	TARGET := x86_64-unknown-linux-musl
else ifeq ($(OSTYPE),Darwin)
	TARGET := x86_64-apple-darwin
else
$(error "Unsupported OSTYPE: $(OSTYPE)")
endif

BINARY  := procs
VERSION := $(word 3,$(shell grep -m1 "^version" Cargo.toml))
RELEASE := $(BINARY)-$(VERSION)-$(shell echo $(OSTYPE) | tr "[:upper:]" "[:lower:]")

all: release

$(BINARY):
	cargo build --locked --release --target=$(TARGET)

bin:
	mkdir -p $@

bin/$(BINARY): $(BINARY) bin
	cp -f target/$(TARGET)/release/$(BINARY) $@

release: bin/$(BINARY)
	tar -C bin -Jcvf $(RELEASE).tar.xz $(BINARY)

.PHONY: all release

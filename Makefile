BINARY  := deob
DESTDIR := /usr/local/bin
CARGO   := $(shell which cargo)

.PHONY: build install uninstall test clean fmt lint

build:
	$(CARGO) build --release

install:
	install -m 755 target/release/$(BINARY) $(DESTDIR)/$(BINARY)

uninstall:
	rm -f $(DESTDIR)/$(BINARY)

test:
	$(CARGO) test

clean:
	$(CARGO) clean

fmt:
	$(CARGO) fmt --check

lint:
	$(CARGO) clippy -- -D warnings

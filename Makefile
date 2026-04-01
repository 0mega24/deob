BINARY  := deob
DESTDIR := /usr/local/bin
CARGO   := $(shell which cargo)

.PHONY: build install uninstall test clean

build:
	$(CARGO) build --release

# install only copies — run 'make build' first, then 'sudo make install'
install:
	install -m 755 target/release/$(BINARY) $(DESTDIR)/$(BINARY)

uninstall:
	rm -f $(DESTDIR)/$(BINARY)

test:
	$(CARGO) test

clean:
	$(CARGO) clean

SHELL                      := /bin/bash

LIB												 := sformat-dynamic

CARGO_BIN                  := $(shell which cargo)

WORKSPACE_CARGO_FILE       := Cargo.toml

README.md: README.tpl $(WORKSPACE_CARGO_FILE) $(LIB)/Cargo.toml $(LIB)/src/lib.rs
	$(CARGO_BIN) readme -r $(LIB) -t ../README.tpl -o ../$@

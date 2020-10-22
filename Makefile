schema:
	make -C types schema

fmt:
	cd contracts/toCKB-typescript && cargo fmt --all
	cd contracts/toCKB-lockscript && cargo fmt --all
	cd tests && cargo fmt --all

build:
	capsule build

test:
	capsule test

ci: fmt build test

cli:
	cargo +1.47.0 build

.PHONY: fmt build test ci schema cli

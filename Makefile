watch:
	watchexec -w contracts/toCKB-typescript/src -- 'make fmt && capsule build'

schema:
	moleculec --language rust --schema-file contracts/toCKB-typescript/src/utils/types/schemas/basic.mol > contracts/toCKB-typescript/src/utils/types/generated/basic.rs
	moleculec --language rust --schema-file contracts/toCKB-typescript/src/utils/types/schemas/toCKB_cell_data.mol > contracts/toCKB-typescript/src/utils/types/generated/toCKB_cell_data.rs
	moleculec --language rust --schema-file contracts/toCKB-typescript/src/utils/types/schemas/btc_difficulty.mol > contracts/toCKB-typescript/src/utils/types/generated/btc_difficulty.rs
	moleculec --language rust --schema-file contracts/toCKB-typescript/src/utils/types/schemas/mint_xt_witness.mol > contracts/toCKB-typescript/src/utils/types/generated/mint_xt_witness.rs
	cp contracts/toCKB-typescript/src/utils/types/generated/*.rs tests/src/toCKB_typescript/utils/types/generated

fmt:
	cd contracts/toCKB-typescript && cargo fmt --all
	cd contracts/toCKB-lockscript && cargo fmt --all
	cd tests && cargo fmt --all

build:
	capsule build

test:
	capsule test

ci: fmt build test

.PHONY: fmt build test ci
.PHONY: schema watch
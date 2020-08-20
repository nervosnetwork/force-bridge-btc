watch:
	watchexec -w contracts/toCKB-typescript/src -- 'make fmt && capsule build'

schema:
	moleculec --language rust --schema-file contracts/toCKB-typescript/src/utils/types/schemas/toCKB_cell_data.mol > contracts/toCKB-typescript/src/utils/types/generated/toCKB_cell_data.rs

fmt:
	cd contracts/toCKB-typescript && cargo fmt

.PHONY: schema fmt watch

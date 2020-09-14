SCHEMA_PATH := schemas
SCHEMA_DEST_PATH := src/generated

schema:
	moleculec --language rust --schema-file ${SCHEMA_PATH}/basic.mol > ${SCHEMA_DEST_PATH}/basic.rs
	moleculec --language rust --schema-file ${SCHEMA_PATH}/tockb_cell_data.mol > ${SCHEMA_DEST_PATH}/tockb_cell_data.rs
	moleculec --language rust --schema-file ${SCHEMA_PATH}/btc_difficulty.mol > ${SCHEMA_DEST_PATH}/btc_difficulty.rs
	moleculec --language rust --schema-file ${SCHEMA_PATH}/mint_xt_witness.mol > ${SCHEMA_DEST_PATH}/mint_xt_witness.rs
	cargo fmt

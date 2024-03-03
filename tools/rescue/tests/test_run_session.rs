use std::path::Path;

use anyhow::Context;
use diem_config::config::{
    RocksdbConfigs, BUFFERED_STATE_TARGET_ITEMS, DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    NO_OP_STORAGE_PRUNER_CONFIG,
};
use diem_db::DiemDB;
use diem_storage_interface::{state_view::DbStateViewAtVersion, DbReaderWriter};
use rescue::writeset_builder::{build_changeset, GenesisSession};
#[test]

// Run a VM session with a dirty database
// NOTE: there are several implementations of this elsewhere in Diem
// Some are buggy, some don't have exports or APIs needed (DiemDbBootstrapper). Some have issues with async and db locks (DiemDbDebugger).
// so we had to rewrite it.
pub fn libra_run_session() -> anyhow::Result<()> {
    let db = DiemDB::open(
        Path::new("/root/db"),
        true,
        NO_OP_STORAGE_PRUNER_CONFIG, /* pruner */
        RocksdbConfigs::default(),
        false, /* indexer */
        BUFFERED_STATE_TARGET_ITEMS,
        DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    )
    .context("Failed to open DB.")
    .unwrap();

    let db_rw = DbReaderWriter::new(db);
    let v = db_rw.reader.get_latest_version().unwrap();

    let view = db_rw.reader.state_view_at_version(Some(v)).unwrap();

    let _changeset = build_changeset(&view, noop, 1);

    println!("session run sucessfully");
    Ok(())
}

fn noop(_session: &mut GenesisSession) {
    dbg!("hi");
}

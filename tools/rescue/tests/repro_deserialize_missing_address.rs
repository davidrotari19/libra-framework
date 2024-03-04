use std::{path::Path, sync::Arc};

use anyhow::Context;
use diem_config::config::{
    RocksdbConfigs, BUFFERED_STATE_TARGET_ITEMS, DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    NO_OP_STORAGE_PRUNER_CONFIG,
};
use diem_db::DiemDB;
use diem_state_view::TStateView;
use diem_storage_interface::{state_view::DbStateViewAtVersion, DbReaderWriter};

use libra_smoke_tests::libra_smoke::LibraSmoke;
use rescue::writeset_builder::{build_changeset, GenesisSession};

use diem_validator_interface::{DBDebuggerInterface, DebuggerStateView, DiemValidatorInterface};

#[tokio::test]

// Run a VM session with a dirty database
// NOTE: there are several implementations of this elsewhere in Diem
// Some are buggy, some don't have exports or APIs needed (DiemDbBootstrapper). Some have issues with async and db locks (DiemDbDebugger).
// so we had to rewrite it.
pub async fn repro_alt() -> anyhow::Result<()> {
    let mut smoke = LibraSmoke::new(None).await?;
    let marlon_node = smoke.swarm.validators_mut().next().unwrap();
    let swarm_db_path = marlon_node.config().storage.dir();
    marlon_node.stop();

    let db = DiemDB::open(
        swarm_db_path, //Path::new("/root/db"),
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

    let _changeset = build_changeset(&view, repro, 1);

    println!("session run sucessfully");
    Ok(())
}

#[tokio::test]
pub async fn repro_alt_w_debugger_interface() -> anyhow::Result<()> {
    // let mut smoke = LibraSmoke::new(None).await?;
    // let marlon_node = smoke.swarm.validators_mut().next().unwrap();
    // let swarm_db_path = marlon_node.config().storage.dir();
    // marlon_node.stop();

    // let debugger = DBDebuggerInterface::open(swarm_db_path)?;
    let swarm_db_path: &Path = Path::new("/root/db/");
    let debugger = Arc::new(DBDebuggerInterface::open(swarm_db_path)?);

    let version = debugger.get_latest_version().await?;
    dbg!(&version);
    let state_view = DebuggerStateView::new(debugger, version);

    dbg!(&state_view.id());

    let _changeset = build_changeset(&state_view, repro, 1);

    println!("session run sucessfully");
    Ok(())
}

fn _noop(_session: &mut GenesisSession) {
    dbg!("hi");
}

fn repro(session: &mut GenesisSession) {
    session.exec_func("epoch_helper", "get_current_epoch", vec![], vec![]);
    session.exec_func("repro_deserialize", "should_not_abort", vec![], vec![])
}

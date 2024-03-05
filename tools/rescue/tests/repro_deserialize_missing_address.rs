use std::{path::Path, sync::Arc};

use anyhow::Context;
use diem_config::config::{
    RocksdbConfigs, BUFFERED_STATE_TARGET_ITEMS, DEFAULT_MAX_NUM_NODES_PER_LRU_CACHE_SHARD,
    NO_OP_STORAGE_PRUNER_CONFIG,
};
use diem_db::DiemDB;
use diem_debugger::DiemDebugger;
use diem_state_view::TStateView;
use diem_storage_interface::{state_view::DbStateViewAtVersion, DbReaderWriter};

use diem_types::account_config;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use move_core_types::{
    identifier::Identifier,
    language_storage::ModuleId,
    value::{serialize_values, MoveValue},
};
use move_vm_types::gas::UnmeteredGasMeter;
use rescue::writeset_builder::{build_changeset, GenesisSession};

use diem_validator_interface::{DBDebuggerInterface, DebuggerStateView, DiemValidatorInterface};
use libra_types::exports::AccountAddress;

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

// #[tokio::test]
#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
pub async fn repro_deserialize_debugger() -> anyhow::Result<()> {
    // get a clean swarm db with current framework
    let mut smoke = LibraSmoke::new(None).await?;
    let marlon_node = smoke.swarm.validators_mut().next().unwrap();
    marlon_node.stop(); // should safely stop diem-node process, and prevent any DB locks.
    let swarm_db_path = marlon_node.config().storage.dir();
    // or use a fixture db. The error is the same.
    // let swarm_db_path: &Path = Path::new("/root/swarm_db/");

    let debug = DiemDebugger::db(swarm_db_path)?;

    let version = debug.get_latest_version().await?;
    dbg!(&version);
    // OK So far
    // This hangs. Hanging at StorageAdapter::new()
    let s = debug
        .annotate_account_state_at_version("0x1".parse()?, version)
        .await?;
    dbg!(&s.is_some());

    debug.run_session_at_version(version, |session| {
        let r = session.execute_function_bypass_visibility(
            &ModuleId::new(
                account_config::CORE_CODE_ADDRESS,
                Identifier::new("repro_deserialize").unwrap(),
            ),
            &Identifier::new("should_not_abort").unwrap(),
            vec![],
            serialize_values(vec![]),
            &mut UnmeteredGasMeter,
        );
        dbg!(&r.is_ok());

        let addr = MoveValue::Address(AccountAddress::from_hex_literal("0xabc").unwrap());

        let r = session.execute_entry_function(
            &ModuleId::new(
                account_config::CORE_CODE_ADDRESS,
                Identifier::new("repro_deserialize").unwrap(),
            ),
            &Identifier::new("maybe_aborts").unwrap(),
            vec![],
            serialize_values(vec![&addr]),
            &mut UnmeteredGasMeter,
        );
        dbg!(&r.is_ok());

        Ok(())
    })?;

    Ok(())
}

fn _noop(_session: &mut GenesisSession) {
    dbg!("hi");
}

fn repro(session: &mut GenesisSession) {
    session.exec_func("epoch_helper", "get_current_epoch", vec![], vec![]);
    session.exec_func("repro_deserialize", "should_not_abort", vec![], vec![])
}

use diem_debugger::DiemDebugger;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use std::path::PathBuf;

#[tokio::test]
pub async fn repro_debugger_hangs() -> anyhow::Result<()> {
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
    dbg!(&s);

    Ok(())
}

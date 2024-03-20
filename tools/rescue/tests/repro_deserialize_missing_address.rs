use diem_debugger::DiemDebugger;
use diem_types::{account_address::AccountAddress, account_config};
use diem_vm::move_vm_ext::SessionExt;
use libra_smoke_tests::libra_smoke::LibraSmoke;
use move_core_types::{
    identifier::Identifier,
    language_storage::ModuleId,
    value::{serialize_values, MoveValue},
};
use move_vm_types::gas::UnmeteredGasMeter;

#[tokio::test(flavor = "multi_thread", worker_threads = 5)]
pub async fn repro_deserialize_debugger() -> anyhow::Result<()> {
    // The diem-node should be compiled externally to avoid any potential conflicts with the current build
    //get the current path
    let current_path = std::env::current_dir()?;
    //path to diem-node
    let diem_node_path = current_path.join("tests/diem-ghost");
    //Run cargo clear to make sure we have the latest changes
    let _ = std::process::Command::new("cargo")
        .current_dir(&diem_node_path)
        .args(&["clean"])
        .output()
        .expect("failed to execute process");
    //Run cargo build to make sure we have the latest changes
    let _ = std::process::Command::new("cargo")
        .current_dir(&diem_node_path)
        .args(&["build", "--package", "diem-node", "--release"])
        .output()
        .expect("failed to execute process");

    // Get the path diem-node binary
    let diem_node_bin_path = diem_node_path.join("target/release/diem-node");

    //export env var to use release
    std::env::set_var("DIEM_FORGE_NODE_BIN_PATH", diem_node_bin_path);

    // get a clean swarm db with current framework
    let mut smoke = LibraSmoke::new(None).await?;
    let marlon_node = smoke.swarm.validators_mut().next().unwrap();
    //wait for a while to make sure the node is up and running
    //tokio::time::sleep(tokio::time::Duration::from_secs(100)).await;
    marlon_node.stop(); // should safely stop diem-node process, and prevent any DB locks.
    let swarm_db_path = marlon_node.config().storage.dir();
    // or use a fixture db. The error is the same.
    // let swarm_db_path: &Path = Path::new("/root/swarm_db/");

    let debug = DiemDebugger::db(swarm_db_path)?;

    let version = debug.get_latest_version().await?;
    dbg!(&version);
    let rand = AccountAddress::random();
    let addr = MoveValue::Address(rand);


    let _ = debug
        .run_session_at_version(version, |session| {
            // let root = MoveValue::Signer("0x1".parse().unwrap());
            execute_fn(session, "account", "create_account", vec![&addr]);

            // execute_fn(session, "repro_deserialize", "maybe_aborts", vec![&addr]);
            Ok(())
        })
        .expect("could run session");
    std::env::remove_var("DIEM_FORGE_NODE_BIN_PATH");
    Ok(())
}

fn execute_fn(session: &mut SessionExt, module: &str, function: &str, args: Vec<&MoveValue>) {
    let t = session
        .execute_function_bypass_visibility(
            &ModuleId::new(
                account_config::CORE_CODE_ADDRESS,
                Identifier::new(module).unwrap(),
            ),
            &Identifier::new(function).unwrap(),
            vec![],
            serialize_values(args),
            &mut UnmeteredGasMeter,
        )
        .expect("run function");
    dbg!(&t);
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), String> {
    let mut args: Vec<_> = std::env::args().skip(1).collect();
    if cm_trace::run::generate_cli(&args)? {
        return Ok(());
    }
    let retain_store = cm_trace::run::take_retain_store(&mut args);
    if args.len() != 3 || args[0] != "run" {
        return Err("usage: cm-harness generate small|1k|10k|100k OUTPUT.cmt | run TRACE.cmt NEW_OUTPUT_DIRECTORY [--retain-store]".into());
    }
    let trace = cm_trace::run::load(std::path::Path::new(&args[1]))?;
    cm_trace::run::series_with_retention(
        &trace,
        std::path::Path::new(&args[2]),
        &cm_harness::label(),
        cm_trace::D1,
        retain_store,
        cm_harness::CandidateEngine::create,
    )
}

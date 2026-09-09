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
    if args.len() != 4 || args[0] != "run" {
        return Err(
            "usage: cm-legacy-runner run TRACE.cmt NEW_OUTPUT_DIRECTORY single|pipelined|atomic [--retain-store]"
                .into(),
        );
    }
    let mode = cm_legacy_runner::Mode::parse(&args[3])?;
    let trace = cm_trace::run::load(std::path::Path::new(&args[1]))?;
    let label = cm_legacy_runner::label_for(mode);
    cm_trace::run::series_with_retention(
        &trace,
        std::path::Path::new(&args[2]),
        &label,
        cm_legacy_runner::DURABILITY,
        retain_store,
        |p| cm_legacy_runner::Legacy::create(p, mode),
    )
}

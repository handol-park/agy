use agy::scorecard::{load_entries_from_file, render_markdown_table, update_doc_table};
use std::env;
use std::fs;

fn print_usage() {
    eprintln!(
        "Usage:\n  scorecard <input.json> [--output <table.md>] [--update-doc <doc.md>]\n\nExamples:\n  cargo run --bin scorecard -- benchmarks/provider-results.json\n  cargo run --bin scorecard -- benchmarks/provider-results.json --output /tmp/scorecard-table.md\n  cargo run --bin scorecard -- benchmarks/provider-results.json --update-doc docs/provider-scorecard.md"
    );
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let input = args
        .next()
        .ok_or_else(|| "missing input.json path".to_string())?;

    let mut output_path: Option<String> = None;
    let mut update_doc_path: Option<String> = None;

    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--output" => {
                let path = args
                    .next()
                    .ok_or_else(|| "missing value for --output".to_string())?;
                output_path = Some(path);
            }
            "--update-doc" => {
                let path = args
                    .next()
                    .ok_or_else(|| "missing value for --update-doc".to_string())?;
                update_doc_path = Some(path);
            }
            _ => return Err(format!("unknown argument: {flag}")),
        }
    }

    let entries = load_entries_from_file(&input)?;
    let table = render_markdown_table(&entries);

    if let Some(path) = output_path.as_ref() {
        fs::write(&path, &table).map_err(|e| format!("failed writing '{path}': {e}"))?;
    }

    if let Some(path) = update_doc_path.as_ref() {
        let doc = fs::read_to_string(&path).map_err(|e| format!("failed reading '{path}': {e}"))?;
        let updated = update_doc_table(&doc, &table)?;
        fs::write(&path, updated).map_err(|e| format!("failed writing '{path}': {e}"))?;
    }

    if output_path.is_none() && update_doc_path.is_none() {
        println!("{table}");
    }

    Ok(())
}

fn main() {
    if env::args().len() <= 1 {
        print_usage();
        std::process::exit(2);
    }

    if let Err(err) = run() {
        eprintln!("error: {err}");
        print_usage();
        std::process::exit(1);
    }
}

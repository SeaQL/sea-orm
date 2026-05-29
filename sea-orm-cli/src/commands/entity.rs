//! Entity-first commands for sea-orm-cli.

use std::error::Error;
use std::io;

use colored::Colorize;

use crate::commands::subprocess::{
    DiffData, GenerateData, SchemaData, manifest_path, run_subprocess_json,
};

pub fn run_entity_sync(
    dir: &str,
    migration_dir: &str,
    name: Option<&str>,
    database_url: Option<&str>,
    database_schema: Option<&str>,
    allow_dangerous: bool,
    renames: &[String],
    no_confirm: bool,
) -> Result<(), Box<dyn Error>> {
    let manifest = manifest_path(dir);

    let mut diff_args = vec!["diff"];
    if !allow_dangerous {
        diff_args.push("--allow-dangerous=false");
    }

    let (_, diff) =
        run_subprocess_json::<DiffData>(&manifest, &diff_args, database_url, database_schema)
            .map_err(|e| format!("diff failed: {e}"))?;

    let decision = run_sync(diff, name, renames, no_confirm)?;

    match decision {
        SyncDecision::Quit => {
            println!("{}", "Aborted.".yellow());
            return Ok(());
        }
        SyncDecision::Generate {
            schema_hash,
            renames: resolved_renames,
            migration_name: gen_name,
        } => {
            let mut gen_args = vec![
                "generate".to_string(),
                gen_name,
                format!("--migration-dir={migration_dir}"),
                format!("--schema-hash={schema_hash}"),
            ];
            if !allow_dangerous {
                gen_args.push("--allow-dangerous=false".to_string());
            }
            for (table, old, new) in &resolved_renames {
                gen_args.push(format!("--rename={table}.{old}:{new}"));
            }

            let gen_args_ref: Vec<&str> = gen_args.iter().map(String::as_str).collect();

            let (_, result) = run_subprocess_json::<GenerateData>(
                &manifest,
                &gen_args_ref,
                database_url,
                database_schema,
            )
            .map_err(|e| format!("generate failed: {e}"))?;

            print_generate_result(&result);
        }
    }

    Ok(())
}

pub fn run_entity_schema(dir: &str, database_backend: &str) -> Result<(), Box<dyn Error>> {
    let manifest = manifest_path(dir);
    let backend_arg = format!("--database-backend={database_backend}");
    let args = ["schema", backend_arg.as_str()];
    let (_, data) = run_subprocess_json::<SchemaData>(&manifest, &args, None, None)
        .map_err(|e| format!("schema failed: {e}"))?;
    for stmt in &data.statements {
        println!("{stmt}");
    }
    Ok(())
}

pub fn run_entity_init(_dir: &str) -> Result<(), Box<dyn Error>> {
    println!("Entity crate scaffolding is not yet implemented.");
    Ok(())
}

enum SyncDecision {
    Quit,
    Generate {
        schema_hash: String,
        renames: Vec<(String, String, String)>, // (table, old, new)
        migration_name: String,
    },
}

fn run_sync(
    diff: DiffData,
    name: Option<&str>,
    rename_flags: &[String],
    no_confirm: bool,
) -> Result<SyncDecision, Box<dyn Error>> {
    if diff.statements.is_empty() {
        println!("{}", "No schema changes detected. Nothing to migrate.".green());
        return Ok(SyncDecision::Quit);
    }

    println!("{}", format!("Changes ({}):", diff.changes.len()).bold());
    for change in &diff.changes {
        println!("  {} {change}", "-".yellow());
    }

    println!();
    println!("{}", format!("SQL statements ({}):", diff.statements.len()).bold());
    for stmt in &diff.statements {
        println!("  {}", stmt.dimmed());
    }

    if !diff.warnings.is_empty() {
        println!();
        println!("{}", format!("Warnings ({}):", diff.warnings.len()).yellow().bold());
        for w in &diff.warnings {
            println!("  {} {}", format!("[{}]", w.kind).yellow(), w.message);
        }
    }

    if !diff.suggestions.is_empty() {
        println!();
        println!("{}", format!("Suggestions ({}):", diff.suggestions.len()).blue().bold());
        for s in &diff.suggestions {
            println!("  {} {}", format!("[{}]", s.kind).blue(), s.message);
        }
    }

    let mut rename_map: std::collections::HashMap<(String, String), String> =
        std::collections::HashMap::new();
    for flag in rename_flags {
        let (table_col, new) = flag
            .split_once(':')
            .ok_or_else(|| format!("invalid --rename value '{flag}': expected table.old:new"))?;
        let (table, old) = table_col
            .split_once('.')
            .ok_or_else(|| format!("invalid --rename value '{flag}': expected table.old:new"))?;
        rename_map.insert((table.to_string(), old.to_string()), new.to_string());
    }

    let has_rename_flags = !rename_flags.is_empty();
    let schema_hash = diff.schema_hash.clone();
    let mut resolved_renames: Vec<(String, String, String)> = Vec::new();

    if !diff.unresolved.is_empty() {
        println!();
        println!(
            "{}",
            format!("Unresolved renames ({}):", diff.unresolved.len())
                .yellow()
                .bold()
        );
    }

    for unresolved in &diff.unresolved {
        let key = (unresolved.table.clone(), unresolved.removed.clone());

        if let Some(new_col) = rename_map.get(&key) {
            if !unresolved.candidates.contains(new_col) {
                return Err(format!(
                    "--rename {}.{}:{} is invalid: '{}' is not among the candidates: {}",
                    unresolved.table,
                    unresolved.removed,
                    new_col,
                    new_col,
                    unresolved.candidates.join(", ")
                )
                .into());
            }
            resolved_renames.push((
                unresolved.table.clone(),
                unresolved.removed.clone(),
                new_col.clone(),
            ));
        } else if has_rename_flags {
            return Err(format!(
                "unresolved rename for {}.{} (candidates: {}): provide --rename={}.{}:<new_col>",
                unresolved.table,
                unresolved.removed,
                unresolved.candidates.join(", "),
                unresolved.table,
                unresolved.removed,
            )
            .into());
        } else {
            println!(
                "  Table {}: column {} was removed.",
                format!("'{}'", unresolved.table).bold(),
                format!("'{}'", unresolved.removed).yellow()
            );
            println!("  {}", "Candidates for rename:".bold());
            for (i, c) in unresolved.candidates.iter().enumerate() {
                println!("    {}) {}", i + 1, c.cyan());
            }
            println!(
                "    {}) {}",
                (unresolved.candidates.len() + 1).to_string().red(),
                "drop (treat as a plain column drop)".red()
            );

            let choice = prompt_rename_choice(&unresolved.candidates)?;
            if let Some(new_col) = choice {
                resolved_renames.push((
                    unresolved.table.clone(),
                    unresolved.removed.clone(),
                    new_col,
                ));
            }
        }
    }

    let migration_name = match name {
        Some(n) => n.to_string(),
        None => {
            print!("{}", "Migration name (e.g. add_users): ".bold());
            io::Write::flush(&mut io::stdout())?;
            let mut input = String::new();
            io::BufRead::read_line(&mut io::stdin().lock(), &mut input)?;
            let input = input.trim().to_string();
            if input.is_empty() {
                return Err("migration name cannot be empty".into());
            }
            input
        }
    };

    if !no_confirm {
        print!(
            "{}",
            format!("Generate migration '{migration_name}'? [Y/n]: ").bold()
        );
        io::Write::flush(&mut io::stdout())?;
        let mut input = String::new();
        io::BufRead::read_line(&mut io::stdin().lock(), &mut input)?;
        let input = input.trim().to_lowercase();
        if input == "n" || input == "no" {
            return Ok(SyncDecision::Quit);
        }
    }

    Ok(SyncDecision::Generate {
        schema_hash,
        renames: resolved_renames,
        migration_name,
    })
}

fn prompt_rename_choice(candidates: &[String]) -> Result<Option<String>, Box<dyn Error>> {
    let drop_option = candidates.len() + 1;
    loop {
        print!("{}", format!("  Choice [1-{drop_option}]: ").bold());
        io::Write::flush(&mut io::stdout())?;
        let mut input = String::new();
        io::BufRead::read_line(&mut io::stdin().lock(), &mut input)?;
        let input = input.trim();
        match input.parse::<usize>() {
            Ok(n) if n >= 1 && n <= candidates.len() => {
                return Ok(Some(candidates[n - 1].clone()));
            }
            Ok(n) if n == drop_option => {
                return Ok(None);
            }
            _ => {
                println!(
                    "  {}",
                    format!("Please enter a number between 1 and {drop_option}.").yellow()
                );
            }
        }
    }
}

fn print_generate_result(result: &GenerateData) {
    println!();
    println!(
        "  {} {}",
        "Migration generated:".green().bold(),
        result.migration_name.bold()
    );
    println!("  File: {}", result.filepath.dimmed());
    if !result.changes.is_empty() {
        println!("  {}", format!("Changes ({}):", result.changes.len()).bold());
        for change in &result.changes {
            println!("    {} {change}", "+".green());
        }
    }
    println!();
}

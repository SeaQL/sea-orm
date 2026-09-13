//! Entity-first commands for sea-orm-cli.

use std::error::Error;
use std::io;

use colored::Colorize;

use crate::cli::EntitySyncArgs;
use crate::commands::subprocess::{
    AssumedRenameJson, AssumedTableMoveJson, DiffData, GenerateData, SchemaData, manifest_path,
    run_subprocess_json,
};

pub fn run_entity_sync(args: &EntitySyncArgs) -> Result<(), Box<dyn Error>> {
    let manifest = manifest_path(&args.dir);
    let database_url = args.database_url.as_deref();
    let database_schema = args.database_schema.as_deref();

    let diff_args = ["diff"];

    let (_, diff) =
        run_subprocess_json::<DiffData>(&manifest, &diff_args, database_url, database_schema)
            .map_err(|e| format!("diff failed: {e}"))?;

    let plan = match run_sync(diff, args)? {
        SyncDecision::Quit => {
            println!("{}", "Aborted.".yellow());
            return Ok(());
        }
        SyncDecision::Generate(plan) => plan,
    };

    let migration_dir = &args.migration_dir;
    let mut gen_args = vec![
        "generate".to_string(),
        plan.migration_name,
        format!("--migration-dir={migration_dir}"),
        format!("--schema-hash={}", plan.schema_hash),
    ];
    for rename in &plan.renames {
        gen_args.push(format!("--rename={}:{}", rename.column, rename.new_name));
    }
    for column in &plan.rejected_renames {
        gen_args.push(format!("--reject={column}"));
    }
    for idx in &plan.excluded_statements {
        gen_args.push(format!("--exclude={idx}"));
    }
    for table in &plan.rejected_table_moves {
        gen_args.push(format!("--reject-table={table}"));
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

/// A column, addressed as `table.column`. The table is schema-qualified
/// (`schema.table.column`) whenever the entity carries a `schema_name`, which
/// is why every parse of this form splits on the *last* dot.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ColumnRef {
    table: String,
    column: String,
}

impl std::fmt::Display for ColumnRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.table, self.column)
    }
}

/// A rename settled during sync — either from a `--rename` flag or answered
/// at the prompt.
#[derive(Debug, Clone)]
struct ResolvedRename {
    /// The removed column.
    column: ColumnRef,
    /// The added column it was renamed to.
    new_name: String,
}

impl std::str::FromStr for ResolvedRename {
    type Err = String;

    /// Parses the `--rename TABLE.OLD:NEW` flag format.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || format!("invalid --rename value '{s}': expected table.old:new");
        let (table_old, new_name) = s.split_once(':').ok_or_else(invalid)?;
        let (table, column) = table_old.rsplit_once('.').ok_or_else(invalid)?;
        if table.is_empty() || column.is_empty() || new_name.is_empty() {
            return Err(invalid());
        }
        Ok(Self {
            column: ColumnRef {
                table: table.to_owned(),
                column: column.to_owned(),
            },
            new_name: new_name.to_owned(),
        })
    }
}

/// What the interactive review decided to reject, addressed the way the
/// `generate` subcommand expects it back.
#[derive(Debug, Default)]
struct ReviewOutcome {
    /// Assumed column renames the user rejected — each becomes a plain
    /// DROP + ADD instead.
    rejected_renames: Vec<ColumnRef>,
    /// Positions in the diff's statement list to leave out entirely.
    excluded_statements: Vec<usize>,
    /// Old (schema-qualified) names of table moves the user rejected — each
    /// becomes a plain CREATE + DROP instead.
    rejected_table_moves: Vec<String>,
}

/// Everything `generate` needs once the user has settled every open question.
#[derive(Debug)]
struct GeneratePlan {
    schema_hash: String,
    migration_name: String,
    renames: Vec<ResolvedRename>,
    rejected_renames: Vec<ColumnRef>,
    excluded_statements: Vec<usize>,
    rejected_table_moves: Vec<String>,
}

enum SyncDecision {
    Quit,
    Generate(GeneratePlan),
}

fn run_sync(diff: DiffData, args: &EntitySyncArgs) -> Result<SyncDecision, Box<dyn Error>> {
    if diff.statements.is_empty() {
        println!(
            "{}",
            "No schema changes detected. Nothing to migrate.".green()
        );
        return Ok(SyncDecision::Quit);
    }

    println!("{}", format!("Changes ({}):", diff.changes.len()).bold());
    for change in &diff.changes {
        println!("  {} {change}", "-".yellow());
    }

    println!();
    println!(
        "{}",
        format!("SQL statements ({}):", diff.statements.len()).bold()
    );
    for stmt in &diff.statements {
        println!("  {}", stmt.dimmed());
    }

    if !diff.warnings.is_empty() {
        println!();
        println!(
            "{}",
            format!("Warnings ({}):", diff.warnings.len())
                .yellow()
                .bold()
        );
        for w in &diff.warnings {
            println!("  {} {}", format!("[{}]", w.kind).yellow(), w.message);
        }
    }

    if !diff.suggestions.is_empty() {
        println!();
        println!(
            "{}",
            format!("Suggestions ({}):", diff.suggestions.len())
                .blue()
                .bold()
        );
        for s in &diff.suggestions {
            println!("  {} {}", format!("[{}]", s.kind).blue(), s.message);
        }
    }

    let mut rename_map: std::collections::HashMap<ColumnRef, String> =
        std::collections::HashMap::new();
    for flag in &args.renames {
        let rename: ResolvedRename = flag.parse()?;
        rename_map.insert(rename.column, rename.new_name);
    }

    let has_rename_flags = !args.renames.is_empty();
    let schema_hash = diff.schema_hash.clone();
    let mut resolved_renames: Vec<ResolvedRename> = Vec::new();

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
        let column = ColumnRef {
            table: unresolved.table.clone(),
            column: unresolved.removed.clone(),
        };

        if let Some(new_name) = rename_map.get(&column) {
            if !unresolved.candidates.contains(new_name) {
                return Err(format!(
                    "--rename {column}:{new_name} is invalid: '{new_name}' is not among the candidates: {}",
                    unresolved.candidates.join(", ")
                )
                .into());
            }
            resolved_renames.push(ResolvedRename {
                column: column.clone(),
                new_name: new_name.clone(),
            });
        } else if has_rename_flags {
            return Err(format!(
                "unresolved rename for {column} (candidates: {}): provide --rename={column}:<new_col>",
                unresolved.candidates.join(", "),
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
            if let Some(new_name) = choice {
                resolved_renames.push(ResolvedRename {
                    column: column.clone(),
                    new_name,
                });
            }
        }
    }

    let Some(review) = run_change_review(&diff, args.review_all)? else {
        return Ok(SyncDecision::Quit);
    };

    let migration_name = match args.name.as_deref() {
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

    if !args.no_confirm {
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

    Ok(SyncDecision::Generate(GeneratePlan {
        schema_hash,
        migration_name,
        renames: resolved_renames,
        rejected_renames: review.rejected_renames,
        excluded_statements: review.excluded_statements,
        rejected_table_moves: review.rejected_table_moves,
    }))
}

/// Walk the reviewable changes one at a time, prompting `[y/n/b/q]` for each.
/// By default only assumed (auto-applied) changes — column renames and table
/// renames/schema-moves — are reviewed; with `review_all`, every change in
/// `diff.changes` is. A table move may span two statements (rename + schema
/// move); both are reviewed together as one item, keyed by the first index.
/// Returns `None` when the user quits.
fn run_change_review(
    diff: &DiffData,
    review_all: bool,
) -> Result<Option<ReviewOutcome>, Box<dyn Error>> {
    let assumed_by_index: std::collections::HashMap<usize, &AssumedRenameJson> = diff
        .assumed
        .iter()
        .map(|a| (a.statement_index, a))
        .collect();

    // Table moves may produce a second statement (rename + schema move).
    // That second index is never independently reviewable — it's folded
    // into the decision made at the move's first (primary) index.
    let table_move_by_primary_index: std::collections::HashMap<usize, &AssumedTableMoveJson> = diff
        .table_moves
        .iter()
        .filter_map(|m| m.statement_indices.first().map(|&idx| (idx, m)))
        .collect();
    let secondary_table_move_indices: std::collections::HashSet<usize> = diff
        .table_moves
        .iter()
        .flat_map(|m| m.statement_indices.iter().skip(1).copied())
        .collect();

    let queue: Vec<usize> = if review_all {
        (0..diff.changes.len())
            .filter(|idx| !secondary_table_move_indices.contains(idx))
            .collect()
    } else {
        let mut idxs: Vec<usize> = assumed_by_index
            .keys()
            .chain(table_move_by_primary_index.keys())
            .copied()
            .collect();
        idxs.sort_unstable();
        idxs
    };

    if queue.is_empty() {
        return Ok(Some(ReviewOutcome::default()));
    }

    println!();
    println!("{}", "Review changes:".bold());

    let mut decisions: Vec<(usize, bool)> = Vec::new();
    let mut pos = 0;
    while pos < queue.len() {
        let idx = queue[pos];
        let assumed = assumed_by_index.get(&idx);
        let table_move = table_move_by_primary_index.get(&idx);

        println!();
        if let Some(a) = assumed {
            println!(
                "  {} rename {}.{} {} {}",
                "[assumed]".yellow(),
                a.table,
                a.from,
                "->".dimmed(),
                a.to
            );
        }
        if let Some(m) = table_move {
            println!(
                "  {} table move {} {} {}",
                "[assumed]".yellow(),
                m.from,
                "->".dimmed(),
                m.to
            );
        }
        println!("  {}", diff.changes[idx]);
        println!("    {}", diff.statements[idx].dimmed());
        if let Some(m) = table_move {
            for &extra_idx in m.statement_indices.iter().skip(1) {
                println!("    {}", diff.statements[extra_idx].dimmed());
            }
        }

        print!("{}", "  [y]es/[n]o/[b]ack/[q]uit: ".bold());
        io::Write::flush(&mut io::stdout())?;
        let mut input = String::new();
        io::BufRead::read_line(&mut io::stdin().lock(), &mut input)?;
        let input = input.trim().to_lowercase();

        match input.as_str() {
            "y" | "yes" => {
                decisions.push((idx, true));
                pos += 1;
            }
            "n" | "no" => {
                decisions.push((idx, false));
                pos += 1;
            }
            "b" | "back" => {
                if decisions.is_empty() {
                    println!("  {}", "Nothing to go back to.".yellow());
                } else {
                    decisions.pop();
                    pos -= 1;
                }
            }
            "q" | "quit" => return Ok(None),
            _ => println!("  {}", "Please enter y, n, b, or q.".yellow()),
        }
    }

    let mut outcome = ReviewOutcome::default();
    for (idx, accepted) in decisions {
        if accepted {
            continue;
        }
        if let Some(a) = assumed_by_index.get(&idx) {
            outcome.rejected_renames.push(ColumnRef {
                table: a.table.clone(),
                column: a.from.clone(),
            });
        } else if let Some(m) = table_move_by_primary_index.get(&idx) {
            outcome.rejected_table_moves.push(m.from.clone());
        } else {
            outcome.excluded_statements.push(idx);
        }
    }

    Ok(Some(outcome))
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
        println!(
            "  {}",
            format!("Changes ({}):", result.changes.len()).bold()
        );
        for change in &result.changes {
            println!("    {} {change}", "+".green());
        }
    }
    println!();
}

use clap::Parser;
use sea_orm::Database;
use sea_orm_pi_spigot::PiSpigot;

#[derive(Parser)]
#[command(name = "pi-spigot", about = "Compute digits of pi with checkpointing")]
struct Cli {
    /// Number of decimal digits to compute (after "3.")
    #[arg(short, long, default_value_t = 100)]
    digits: u32,

    /// Checkpoint every N digits (0 to disable)
    #[arg(short, long, default_value_t = 100)]
    checkpoint: u32,

    /// SQLite database path for persistence
    #[arg(long, default_value = "sqlite://pi.sqlite")]
    db: String,
}

fn main() {
    let cli = Cli::parse();

    let db = Database::connect(&cli.db).expect("Failed to connect to database");

    let spigot = PiSpigot::resume(&db, cli.digits).expect("Failed to initialize");

    println!("Computing {} decimal digits of pi...", cli.digits);
    let result = spigot
        .compute_with_db(&db, cli.checkpoint)
        .expect("Computation failed");

    println!("Finished computing {} digits of pi.", cli.digits);
    println!("3.{result}");
}

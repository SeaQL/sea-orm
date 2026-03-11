use sea_orm::{
    ArrowSchema,
    entity::*,
    prelude::{ChronoUtc, Decimal},
    sea_query::prelude::chrono::Timelike,
};

use log::info;

mod measurement {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "measurement", arrow_schema)]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub recorded_at: ChronoDateTimeUtc,
        pub sensor_id: i32,
        pub temperature: f64,
        #[sea_orm(column_type = "Decimal(Some((10, 4)))")]
        pub voltage: Decimal,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let env = env_logger::Env::default().filter_or("RUST_LOG", "info,sea_orm=info,sqlx=warn");
    env_logger::Builder::from_env(env).init();

    // -----------------------------------------------------------------------
    // Step 1: Generate 100 random rows
    // -----------------------------------------------------------------------
    let base_ts = ChronoUtc::now();
    let base_ts = base_ts
        .with_nanosecond((base_ts.nanosecond() / 1000) * 1000)
        .unwrap(); // truncate to microsecond
    let mut rng = fastrand::Rng::new();

    let models: Vec<measurement::ActiveModel> = (1..=100)
        .map(|i| {
            let offset = std::time::Duration::from_secs(rng.u64(0..86_400));
            let ts = base_ts + offset;
            let sensor_id = rng.i32(100..110);
            let temperature = -10.0 + rng.f64() * 50.0; // -10 .. 40 °C
            let voltage_raw = 30000 + rng.i64(0..5000); // 3.0000 .. 3.5000
            measurement::ActiveModel {
                id: Set(i),
                recorded_at: Set(ts),
                sensor_id: Set(sensor_id),
                temperature: Set(temperature),
                voltage: Set(Decimal::new(voltage_raw, 4)),
            }
        })
        .collect();

    let schema = measurement::Entity::arrow_schema();
    info!("Arrow schema: {schema:?}");

    // -----------------------------------------------------------------------
    // Step 2: Convert to Arrow RecordBatch and write to Parquet
    // -----------------------------------------------------------------------
    let batch = measurement::ActiveModel::to_arrow(&models, &schema)?;
    info!(
        "RecordBatch: {} rows, {} columns",
        batch.num_rows(),
        batch.num_columns()
    );

    let parquet_path = "measurements.parquet";

    {
        let file = std::fs::File::create(parquet_path)?;
        let mut writer = parquet::arrow::ArrowWriter::try_new(file, schema.into(), None)?;
        writer.write(&batch)?;
        writer.close()?;
    }
    info!("Wrote Parquet file: {parquet_path}");

    // -----------------------------------------------------------------------
    // Step 3: Read the Parquet file back into a RecordBatch
    // -----------------------------------------------------------------------
    let file = std::fs::File::open(parquet_path)?;
    let reader =
        parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder::try_new(file)?.build()?;

    let batches: Vec<_> = reader.collect::<Result<_, _>>()?;
    info!("Read {} batch(es) from Parquet", batches.len());

    let read_batch = &batches[0];
    assert_eq!(read_batch.num_rows(), 100);

    // Convert back to ActiveModels
    let restored = measurement::ActiveModel::from_arrow(read_batch)?;
    info!("Restored {} ActiveModels from Parquet", restored.len());

    for (original, restored) in models.iter().zip(restored.iter()) {
        assert_eq!(original, restored, "Roundtrip mismatch");
    }
    info!("Parquet roundtrip verified: all rows match.");

    // -----------------------------------------------------------------------
    // Step 4: Dump into SQLite
    // -----------------------------------------------------------------------

    match std::fs::remove_file("measurements.sqlite") {
        Ok(_) => (),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => panic!("Failed to remove file: {e}"),
    }

    let db = &sea_orm::Database::connect("sqlite://measurements.sqlite")?;

    db.get_schema_builder()
        .register(measurement::Entity)
        .apply(db)?;
    info!("SQLite schema created.");

    measurement::Entity::insert_many(restored).exec(db)?;
    info!("Inserted all rows into SQLite.");

    info!("Done!");
    Ok(())
}

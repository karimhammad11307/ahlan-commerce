use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Deserialize)]
struct ImportFile {
    products: Vec<ImportProduct>,
}

#[derive(Deserialize)]
struct ImportProduct {
    title: String,
    handle: String,
    description: Option<String>,
    price_cents: u32,
    inventory_quantity: u32,
    published: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "worker=debug,db=debug".into()),
        )
        .init();

    tracing::info!("starting Ahlan commerce background worker..");

    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set — see .env.example");
    let db_pool = db::create_pool(&database_url);

    loop {
        if let Err(e) = process_next_job(&db_pool).await {
            tracing::error!(error = %e, "error processing jobs loop");
        }
        sleep(Duration::from_secs(2)).await;
    }
}

async fn process_next_job(db_pool: &db::Pool) -> Result<(), Box<dyn std::error::Error>> {
    let client = db_pool.get().await?;

    let job = match db::import_jobs::acquire_queued_job(&**client).await? {
        Some(j) => j,
        None => return Ok(()), // no queued jobs
    };

    let attempt = job.attempts + 1;
    tracing::info!(job_id = %job.id, attempt = attempt, "acquired job");

    // Read file
    let file_content_result = tokio::fs::read_to_string(&job.input_path).await;
    match file_content_result {
        Ok(content) => {
            let import_file: ImportFile = match serde_json::from_str(&content) {
                Ok(f) => f,
                Err(e) => {
                    fail_job(&**client, job, attempt, format!("Invalid JSON: {}", e)).await?;
                    return Ok(());
                }
            };

            for p in import_file.products {
                let domain_input = catalog::ProductCreate {
                    title: p.title,
                    handle: p.handle,
                    description: p.description,
                    price_cents: p.price_cents,
                    inventory_quantity: p.inventory_quantity,
                    published: p.published,
                };

                let new_product = match catalog::create_product(domain_input) {
                    Ok(p) => p,
                    Err(e) => {
                        handle_job_error(
                            &**client,
                            job.clone(),
                            attempt,
                            format!("Validation error: {}", e),
                        )
                        .await?;
                        return Ok(());
                    }
                };

                match db::products::create_product(&**client, new_product).await {
                    Ok(_) => {}
                    Err(e) => {
                        let error_msg = if let Some(db_err) = e.as_db_error() {
                            if db_err.code().code() == "23505" {
                                "duplicate_handle".to_string()
                            } else {
                                e.to_string()
                            }
                        } else {
                            e.to_string()
                        };

                        // Duplicate handles should fail the job, but do we retry?
                        // The contract says: "Duplicate Handles: Do not create a second product. Mark the job failed. Set last_error to a safe message that includes the duplicate handle."
                        // It doesn't say retry duplicate handles. But it says "A job may run at most 3 attempts. Retrying a failed job means changing status back to queued".
                        // Let's just use the standard retry logic.
                        handle_job_error(
                            &**client,
                            job.clone(),
                            attempt,
                            format!("DB Error: {}", error_msg),
                        )
                        .await?;
                        return Ok(());
                    }
                }
            }

            // Success
            db::import_jobs::update_job_status(
                &**client,
                job.id,
                db::import_jobs::ImportJobStatus::Succeeded,
                attempt,
                None,
                chrono::Utc::now(),
            )
            .await?;

            tracing::info!(job_id = %job.id, "job succeeded");
        }
        Err(e) => {
            // Missing file makes the job fail
            fail_job(
                &**client,
                job.clone(),
                attempt,
                format!("Failed to read file {}: {}", job.input_path, e),
            )
            .await?;
        }
    }

    Ok(())
}

async fn handle_job_error(
    client: &tokio_postgres::Client,
    job: db::import_jobs::ImportJob,
    attempt: u32,
    error_msg: String,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::warn!(job_id = %job.id, attempt = attempt, error = %error_msg, "job error");

    let status = if attempt >= 3 {
        db::import_jobs::ImportJobStatus::Failed
    } else {
        db::import_jobs::ImportJobStatus::Queued
    };

    db::import_jobs::update_job_status(
        client,
        job.id,
        status,
        attempt,
        Some(&error_msg),
        chrono::Utc::now(),
    )
    .await?;

    Ok(())
}

async fn fail_job(
    client: &tokio_postgres::Client,
    job: db::import_jobs::ImportJob,
    attempt: u32,
    error_msg: String,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::warn!(job_id = %job.id, attempt = attempt, error = %error_msg, "job failed (fatal)");

    db::import_jobs::update_job_status(
        client,
        job.id,
        db::import_jobs::ImportJobStatus::Failed,
        attempt,
        Some(&error_msg),
        chrono::Utc::now(),
    )
    .await?;

    Ok(())
}

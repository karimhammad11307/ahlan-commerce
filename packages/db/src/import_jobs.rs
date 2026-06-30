use crate::convert_time_to_chrono;
use crate::cornucopia::queries;
use chrono::{DateTime, Utc};
use time::OffsetDateTime;
use tokio_postgres::Error as PostgresError;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportJobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
}

impl ImportJobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImportJobStatus::Queued => "queued",
            ImportJobStatus::Running => "running",
            ImportJobStatus::Succeeded => "succeeded",
            ImportJobStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "queued" => ImportJobStatus::Queued,
            "running" => ImportJobStatus::Running,
            "succeeded" => ImportJobStatus::Succeeded,
            "failed" => ImportJobStatus::Failed,
            _ => ImportJobStatus::Failed, // Default fallback
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImportJob {
    pub id: Uuid,
    pub status: ImportJobStatus,
    pub input_path: String,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn map_job_row(
    row: (
        Uuid,
        String,
        String,
        i32,
        String,
        OffsetDateTime,
        OffsetDateTime,
    ),
) -> ImportJob {
    ImportJob {
        id: row.0,
        status: ImportJobStatus::from_str(&row.1),
        input_path: row.2,
        attempts: row.3 as u32,
        last_error: if row.4.is_empty() { None } else { Some(row.4) },
        created_at: convert_time_to_chrono(row.5),
        updated_at: convert_time_to_chrono(row.6),
    }
}

pub async fn insert_import_job(
    client: &impl cornucopia_client::GenericClient,
    id: Uuid,
    status: ImportJobStatus,
    input_path: &str,
    attempts: u32,
    last_error: Option<&str>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
) -> Result<ImportJob, PostgresError> {
    let created_at_time = OffsetDateTime::from_unix_timestamp(created_at.timestamp()).unwrap();
    let updated_at_time = OffsetDateTime::from_unix_timestamp(updated_at.timestamp()).unwrap();

    let last_error_str = last_error.unwrap_or_default();

    let res = queries::import_jobs::insert_import_job(
        client,
        &id,
        &status.as_str(),
        &input_path,
        &(attempts as i32),
        &last_error_str,
        &created_at_time,
        &updated_at_time,
    )
    .await?;

    Ok(map_job_row(res))
}

pub async fn acquire_queued_job(
    client: &impl cornucopia_client::GenericClient,
) -> Result<Option<ImportJob>, PostgresError> {
    let res = queries::import_jobs::acquire_queued_job(client).await?;
    Ok(res.map(map_job_row))
}

pub async fn update_job_status(
    client: &impl cornucopia_client::GenericClient,
    id: Uuid,
    status: ImportJobStatus,
    attempts: u32,
    last_error: Option<&str>,
    updated_at: DateTime<Utc>,
) -> Result<(), PostgresError> {
    let updated_at_time = OffsetDateTime::from_unix_timestamp(updated_at.timestamp()).unwrap();
    let last_error_str = last_error.unwrap_or_default();

    queries::import_jobs::update_job_status(
        client,
        &status.as_str(),
        &(attempts as i32),
        &last_error_str,
        &updated_at_time,
        &id,
    )
    .await?;

    Ok(())
}

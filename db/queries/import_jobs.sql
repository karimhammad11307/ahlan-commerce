--! insert_import_job(id, status, input_path, attempts, last_error, created_at, updated_at)
INSERT INTO import_jobs (
    id, status, input_path, attempts, last_error, created_at, updated_at
) VALUES (
    $1, $2, $3, $4, $5, $6, $7
)
RETURNING *;

--! acquire_queued_job() ?
UPDATE import_jobs
SET status = 'running', updated_at = now()
WHERE id = (
    SELECT id
    FROM import_jobs
    WHERE status = 'queued'
    ORDER BY created_at ASC
    FOR UPDATE SKIP LOCKED
    LIMIT 1
)
RETURNING *;

--! update_job_status(status, attempts, last_error, updated_at, id)
UPDATE import_jobs
SET status = $1, attempts = $2, last_error = $3, updated_at = $4
WHERE id = $5;

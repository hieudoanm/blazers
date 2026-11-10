# Cron jobs

Cron jobs fire HTTP requests on a schedule — PocketBase's "cron" equivalent
plus what Supabase Edge Functions cron does, minus the function runtime.
Blazers doesn't run shell commands; a "job" is an outbound HTTP call.

## Model

```json
{
  "id": "...",
  "name": "nightly-report",
  "schedule": "0 0 2 * * *",
  "command": "https://reports.example.com/generate",
  "method": "POST",
  "headers": "{\"Authorization\":\"Bearer ...\"}",
  "body": "{\"report\":\"daily\"}",
  "is_active": true,
  "last_run_at": "...",
  "last_run_status": "success",
  "created_at": "...",
  "updated_at": "..."
}
```

- `schedule` is a standard 6-field cron expression with **seconds**
  (`sec min hour dom mon dow`), parsed by the `cron` crate — e.g.
  `*/30 * * * * *` every 30 s, `0 0 9 * * 1-5` weekdays at 09:00.
- `command` is the target URL; `method` is `GET`/`POST`/`PUT`/`PATCH`/`DELETE`
  (anything else falls back to GET).
- `headers` is a JSON object string; `body` is sent for
  POST/PUT/PATCH.

## Execution

A scheduler task ticks every 30 s, finds jobs whose cron expression is due,
and executes them (`cronjobs::execute_cron_job`). Each run writes a
`cron_job_logs` row:

```json
{
  "id": "...",
  "cronjob_id": "...",
  "started_at": "...",
  "finished_at": "...",
  "duration_ms": 142,
  "status": "success",
  "output": "...",
  "error": ""
}
```

`last_run_at` / `last_run_status` on the job mirror the most recent run.

## REST

| Method   | Endpoint                  | Notes                      |
| -------- | ------------------------- | -------------------------- |
| `GET`    | `/api/cronjobs`           | list                       |
| `POST`   | `/api/cronjobs`           | create                     |
| `GET`    | `/api/cronjobs/{id}`      | fetch one                  |
| `PATCH`  | `/api/cronjobs/{id}`      | update (incl. `is_active`) |
| `DELETE` | `/api/cronjobs/{id}`      | delete                     |
| `POST`   | `/api/cronjobs/{id}/run`  | trigger immediately        |
| `GET`    | `/api/cronjobs/{id}/logs` | run history                |

## Webhooks

`cronjob.create`, `cronjob.update`, and `cronjob.delete` events fire
webhooks with `{"cronjob": {...}}` payloads.

## Typical uses

- Hit an internal "recompute" endpoint nightly
- Poll an external API and POST results back into a collection
- Reminder emails via your own mail endpoint

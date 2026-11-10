# Cron Jobs API

Concepts: [Cron jobs](../concepts/cron-jobs.md)

Base: `/api/cronjobs` — admin-level RBAC.

## `GET /api/cronjobs`

List all jobs (array of `CronJob`).

## `POST /api/cronjobs`

**Body**

```json
{
  "name": "nightly",
  "schedule": "0 0 2 * * *",
  "command": "https://example.com/job",
  "method": "POST",
  "headers": "{\"Authorization\":\"Bearer x\"}",
  "body": "{\"run\":true}",
  "is_active": true
}
```

`schedule` is a 6-field cron expression **with seconds**
(`sec min hour dom mon dow`), parsed by the `cron` crate.

**Response `201`** — the `CronJob`. Fires `cronjob.create` webhook.

## `GET /api/cronjobs/{id}`

Fetch one. `404` if missing.

## `PATCH /api/cronjobs/{id}`

Update any field (including `is_active` to pause/resume). Fires
`cronjob.update`.

## `DELETE /api/cronjobs/{id}`

`204`. Fires `cronjob.delete`.

## `POST /api/cronjobs/{id}/run` — trigger now

Executes the job immediately (same path as the scheduler). Returns `200`
after kicking off execution.

## `GET /api/cronjobs/{id}/logs`

Run history for this job — array of `CronJobLog`:

```json
{
  "id": "...",
  "cronjob_id": "...",
  "started_at": "...",
  "finished_at": "...",
  "duration_ms": 120,
  "status": "success",
  "output": "...",
  "error": ""
}
```

# Settings API

Admin-only (`*` `admin` role). Base: `/api/settings`.

Settings are free-form key/value pairs persisted in the `_settings` table.
The admin dashboard has a **Settings** page (`/`) for editing them — SMTP
credentials, app URL and backup schedule.
`GET` returns them all as a JSON object; `smtp_pass` is masked as
`"********"`.

## `GET /api/settings`

```json
{ "smtp_host": "smtp.example.com", "smtp_port": "587",
  "smtp_user": "bot@example.com", "smtp_from": "noreply@example.com",
  "app_url": "https://app.example.com",
  "backup_schedule": "0 3 * * *", "backup_keep": "7" }
```

## `PUT /api/settings`

Body is a flat object; each key is upserted. `smtp_pass` is encrypted with the
server's secrets key before storage — never returned by `GET`.

```sh
curl -X PUT http://localhost:8080/api/settings \
  -H "$AUTH" -H 'Content-Type: application/json' \
  -d '{"smtp_host":"smtp.example.com","smtp_port":"587",
       "smtp_user":"bot@example.com","smtp_pass":"hunter2",
       "smtp_from":"noreply@example.com","app_url":"https://app.example.com"}'
```

## `DELETE /api/settings/{key}`

Remove a setting. `204` / `404`.

## Recognized keys

| Key               | Purpose                                                        |
| ----------------- | -------------------------------------------------------------- |
| `smtp_host`       | SMTP server hostname — enables email delivery                  |
| `smtp_port`       | SMTP port (default `587`, STARTTLS)                            |
| `smtp_user`       | SMTP username                                                  |
| `smtp_pass`       | SMTP password (stored encrypted)                               |
| `smtp_from`       | `From:` address (defaults to `smtp_user`)                      |
| `app_url`         | Public base URL used in reset/verify email links               |
| `backup_schedule` | Cron expression for scheduled backups; empty disables         |
| `backup_dir`      | Backup output dir (default `$BLAZERS_DATA/backups`)            |
| `backup_keep`     | Max snapshots retained, oldest pruned first (`0` = keep all)   |

## Scheduled backups

With `backup_schedule` set (e.g. `0 3 * * *` = daily 03:00 UTC), the server
writes `VACUUM INTO` snapshots to `backup_dir`, pruned to `backup_keep`
files. Changes apply within ~30s without restart. Trigger one manually:

```
POST /api/backup/run   →  { "path": ".../blazers-backup-20240101-030000.db" }
```

Download the live DB anytime via `GET /api/backup`.

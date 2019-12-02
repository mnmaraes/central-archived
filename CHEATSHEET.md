# Setting up Postgres

Installing:
- `brew install postgresql`

Starting/Stopping Server:
- `brew services [start|stop] postgresql`

Checking Status:
- `pg_ctl -D /usr/local/var/postgres -l /usr/local/var/postgres/server.log start`
- If errors: `cat /usr/local/var/postgres/server.log`

Upgrading Existing Database:
- `brew postgresql-upgrade-database`

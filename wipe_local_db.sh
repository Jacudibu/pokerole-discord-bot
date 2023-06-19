rm database.sqlite database.sqlite-shm database.sqlite-wal
sqlx database create
sqlx migrate run

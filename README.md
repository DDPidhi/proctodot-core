# Steps to setup:

### Create Database container:
-  Needed only for initial setup
```shell
docker run --name mariadb-proctodot -e MYSQL_ROOT_PASSWORD=secret -e MYSQL_DATABASE=proctodot -p 3306:3306 -d mariadb
```

---

### Install SEA ORM CLI (if not already installed)
-  Needed only for initial setup
```shell
cargo install sea-orm-cli
```

---

### Create environment file
-  Needed only for initial setup
```shell
cp .env.example .env
```

---

### Generate APP_KEY
- Needed only for initial setup
- Copy the generated key and paste it in .env file as APP_KEY
```shell
openssl rand -base64 32
```

---

### Execute migrations
##### Up
```shell
sea-orm-cli migrate up
```
##### Down (Only if needed)
- sea-orm-cli migrate down -n <number-of-migrations-to-revert>
```shell
sea-orm-cli migrate down -n 100
```
---
### Run project:
#### Build
```cargo
cargo build
```

#### Run
```cargo
cargo run --package proctodot --bin proctodot
```

#### Clean (only if needed)
```cargo
cargo clean
```

#### Start DB
```shell
docker container start mariadb-proctodot
```

#### Stop DB
```shell
docker container stop mariadb-proctodot
```

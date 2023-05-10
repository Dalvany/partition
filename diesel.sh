#!/usr/bin/env bash

DATABASE=$1

if [[ $DATABASE != "mysql" ]] && [[ $DATABASE != "postgres" ]] && [[ $DATABASE != "sqlite" ]]; then
  echo "Usage :"
  echo "diesel.sh [mysql|postgres] --database-url <DATABASE_URL> ...other diesel arguments..."
  echo
  echo "Example ./diesel.sh mysql --database-url mysql://partition:partition@127.0.0.1:3306/partition migration run"
  exit 1
fi

echo "diesel --config-file migrations/$DATABASE.toml ${@:2}"
diesel --config-file migrations/"$DATABASE".toml "${@:2}"
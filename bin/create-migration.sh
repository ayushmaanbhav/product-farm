#!/usr/bin/env bash

NOW=$(date +%s)
MIGRATION_NAME=""
MIGRATIONS_DIR='database/src/main/resources/db/changelog/migrations'

print_usage() {
  printf "Usage: ..."
  printf "\n"
  printf " bin/create-migration.sh -n create-table-mandates"
  printf "\n"
}

while getopts 'n:' flag; do
  case "${flag}" in
  n) MIGRATION_NAME="${OPTARG}" ;;
  *)
    print_usage
    exit 1
    ;;
  esac
done

if [ -z "$MIGRATION_NAME" ]; then
  print_usage
  exit 1
else
  FILE_NAME=${MIGRATIONS_DIR}/"changelog-version"-${NOW}-${MIGRATION_NAME}.xml
  USER_NAME=$(git config user.name)
  touch "${FILE_NAME}"

  echo '<databaseChangeLog
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  xmlns="http://www.liquibase.org/xml/ns/dbchangelog"
  xsi:schemaLocation="http://www.liquibase.org/xml/ns/dbchangelog
         http://www.liquibase.org/xml/ns/dbchangelog/dbchangelog-3.1.xsd">

  <changeSet id="'"${NOW}"'" author="'"${USER_NAME}"'">
  </changeSet>
  </databaseChangeLog>' >"${FILE_NAME}"
  echo "Created migration" "${MIGRATION_NAME}"".xml"
fi
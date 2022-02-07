# Product-FARM
## Product Functionality, Attribute and Rule Management System


## Local setup
#### DB setup
  - install postgres
  - create a user `postgres` with password `admin`
  - create db `template`


#### Install env plugin for intelliJ
 - https://plugins.jetbrains.com/plugin/7861-envfile

#### Run Service
 - Use Run Configuration to create an Application for `database` module
    - add local.env as envfile
 - Run this, this should migrate all of the migrations present
 - Use Run Configuration to create an Application for `manager` module
    - add local.env as envfile
 - Running this should run a local api server

#### Test
- source bin/env-setup.sh; mvn test

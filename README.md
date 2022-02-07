# Template Service


## local setup
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

#### Changes to be done in template repository
- Add SONAR-PROJECT-TOKEN in main pom.xml's property. Refer: https://navihq.atlassian.net/wiki/spaces/NG/pages/414875864/SonarQube+Integration
- Refactor `template` package to the new service name.
- Refactor `TemplateServiceApplication` file according to new service.
- Add new service name in `artifactId` of pom.xml, database/pom.xml & manager/pom.xml.
- Refactor service-name in `appender.console.layout.serviceName` of manager/src/main/resources/log4j2.properties.
- Add excluded file in manager/pom.xml for test cases build exclusions.
- Edit projectName in sonar-analysis.yml.
- Edit configurations in local.env & application.properties of manager.
- For setting up the required pipelines for the new service, kindly refer to: https://navihq.atlassian.net/wiki/spaces/~463086553/pages/386728319/New+Service+Setup+Navi-GI. 


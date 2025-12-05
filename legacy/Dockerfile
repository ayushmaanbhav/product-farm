FROM 193044292705.dkr.ecr.ap-south-1.amazonaws.com/common/maven:3-jdk-11 as builder
ARG ARTIFACT_VERSION=0.0.1-SNAPSHOT
RUN mkdir -p /build
WORKDIR /build
COPY . /build
RUN wget -O elastic-apm.jar https://repo1.maven.org/maven2/co/elastic/apm/elastic-apm-agent/1.15.0/elastic-apm-agent-1.15.0.jar
RUN mvn clean verify -DskipTests -Dartifact.version=${ARTIFACT_VERSION}

FROM 193044292705.dkr.ecr.ap-south-1.amazonaws.com/common/openjdk:11.0.5-jre-slim
ARG ARTIFACT_VERSION=0.0.1-SNAPSHOT
RUN mkdir -p /usr/local
WORKDIR /usr/local/
COPY --from=0 /build/elastic-apm.jar /usr/local/elastic-apm.jar
COPY --from=0 /build/manager/src/main/resources/elasticapm.properties /usr/local/elasticapm.properties
COPY --from=0 /build/manager/target/manager-${ARTIFACT_VERSION}.jar /usr/local/service.jar
COPY --from=0 /build/database/target/database-${ARTIFACT_VERSION}.jar /usr/local/database.jar
RUN adduser --system --uid 4000 --disabled-password app-user && chown -R 4000:4000 /usr/local && chmod -R g+w /usr/local
USER 4000
CMD ["bash", "-c", "java ${JVM_OPTS} -javaagent:/usr/local/elastic-apm.jar -jar /usr/local/service.jar"]
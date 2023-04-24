package io.github.ayushmaanbhav.productFarm.config

import io.github.ayushmaanbhav.jsonLogic.JsonLogicEngine
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.rule.domain.ruleEngine.config.Config
import org.apache.logging.log4j.kotlin.Logging
import org.springframework.context.annotation.Bean
import org.springframework.context.annotation.Configuration

@Configuration
class JsonLogicEngineConfig : Logging {
    @Bean
    fun jsonLogicEngine(config: Config): JsonLogicEngine {
        return JsonLogicEngine.Builder()
            .addLogger { any -> logger.debug("json logic log : $any") }
            .addStandardConfig(StandardLogicOperationConfig(config.mathContext)).build()
    }
}

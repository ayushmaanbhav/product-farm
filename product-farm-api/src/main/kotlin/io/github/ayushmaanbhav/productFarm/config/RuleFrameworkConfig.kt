package io.github.ayushmaanbhav.productFarm.config

import com.fasterxml.jackson.databind.ObjectMapper
import io.github.ayushmaanbhav.jsonLogic.config.MathContext
import io.github.ayushmaanbhav.rule.domain.ruleEngine.config.Config
import io.github.ayushmaanbhav.rule.domain.ruleEngine.model.CachePolicy
import org.springframework.context.annotation.Bean
import org.springframework.context.annotation.Configuration

@Configuration
class RuleFrameworkConfig {
    @Bean
    fun ruleFrameworkConfig(): Config {
        return RuleFrameworkConfig()
    }

    data class RuleFrameworkConfig(
        override val mathContext: MathContext = MathContext.DEFAULT,
        override val objectMapper: ObjectMapper = Config.objectMapperBuilder().build(),
        override val cachePolicy: CachePolicy = Config.DEFAULT_USE_CACHE_POLICY,
        override val maxRuleDgCacheSize: Long = Config.DEFAULT_MAX_CACHE_SIZE,
        override val maxQueryCacheSize: Long = Config.DEFAULT_MAX_CACHE_SIZE
    ) : Config
}

package io.github.ayushmaanbhav.ruleEngine.config

import com.fasterxml.jackson.annotation.JsonInclude
import com.fasterxml.jackson.core.JsonGenerator
import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.databind.ObjectMapper
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule
import com.fasterxml.jackson.module.kotlin.KotlinFeature
import com.fasterxml.jackson.module.kotlin.KotlinModule
import io.github.ayushmaanbhav.jsonLogic.config.MathContext
import io.github.ayushmaanbhav.ruleEngine.model.CachePolicy
import org.springframework.http.converter.json.Jackson2ObjectMapperBuilder

interface Config {
    val mathContext: MathContext
    val objectMapper: ObjectMapper
    val cachePolicy: CachePolicy
    val maxRuleDgCacheSize: Long
    val maxQueryCacheSize: Long

    companion object {
        val DEFAULT_USE_CACHE_POLICY = CachePolicy.DISABLED
        const val DEFAULT_MAX_CACHE_SIZE = -1L // no-limit
        fun objectMapperBuilder(): Jackson2ObjectMapperBuilder {
            val kotlinModule = KotlinModule.Builder()
                .enable(KotlinFeature.StrictNullChecks)
                .build()
            return Jackson2ObjectMapperBuilder.json()
                .serializationInclusion(JsonInclude.Include.NON_NULL)
                .featuresToEnable(
                    JsonGenerator.Feature.WRITE_BIGDECIMAL_AS_PLAIN,
                    DeserializationFeature.USE_BIG_DECIMAL_FOR_FLOATS,
                    DeserializationFeature.USE_BIG_INTEGER_FOR_INTS
                )
                .featuresToDisable(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES)
                .modules(JavaTimeModule())
                .modules(kotlinModule)
        }
    }
}

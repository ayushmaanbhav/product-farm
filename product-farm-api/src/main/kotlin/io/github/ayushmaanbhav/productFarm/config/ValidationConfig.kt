package io.github.ayushmaanbhav.productFarm.config

import jakarta.validation.Validator
import org.springframework.boot.autoconfigure.orm.jpa.HibernatePropertiesCustomizer
import org.springframework.context.annotation.Bean
import org.springframework.context.annotation.Configuration
import org.springframework.validation.beanvalidation.LocalValidatorFactoryBean

@Configuration
class ValidationConfig {
    @Bean
    fun validator(): Validator {
        return LocalValidatorFactoryBean()
    }
    
    @Bean
    fun hibernatePropertiesCustomizer(validator: Validator): HibernatePropertiesCustomizer {
        return HibernatePropertiesCustomizer { hibernateProperties ->
            hibernateProperties["javax.persistence.validation.factory"] = validator
        }
    }
}

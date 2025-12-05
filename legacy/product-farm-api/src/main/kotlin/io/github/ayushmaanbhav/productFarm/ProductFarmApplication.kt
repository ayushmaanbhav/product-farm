package io.github.ayushmaanbhav.productFarm

import com.github.lkqm.spring.api.version.EnableApiVersioning
import org.springframework.boot.autoconfigure.SpringBootApplication
import org.springframework.boot.autoconfigure.domain.EntityScan
import org.springframework.boot.runApplication
import org.springframework.context.annotation.ComponentScan
import org.springframework.data.jpa.repository.config.EnableJpaRepositories

@SpringBootApplication
@ComponentScan("io.github.ayushmaanbhav")
@EntityScan("io.github.ayushmaanbhav.productFarm")
@EnableJpaRepositories("io.github.ayushmaanbhav.productFarm")
@EnableApiVersioning
class ProductFarmApplication {
    companion object {
        @JvmStatic
        fun main(args: Array<String>) {
            runApplication<ProductFarmApplication>(*args)
        }
    }
}

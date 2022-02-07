package com.navi.insurance.productFarm

import org.springframework.boot.autoconfigure.SpringBootApplication
import org.springframework.boot.autoconfigure.domain.EntityScan
import org.springframework.boot.runApplication
import org.springframework.context.annotation.ComponentScan
import org.springframework.data.jpa.repository.config.EnableJpaRepositories

@SpringBootApplication
@ComponentScan("com.navi.insurance")
@EntityScan("com.navi.insurance.productFarm")
@EnableJpaRepositories("com.navi.insurance.productFarm")
class ProductFarmApplication {
    companion object {
        @JvmStatic
        fun main(args: Array<String>) {
            runApplication<ProductFarmApplication>(*args)
        }
    }
}
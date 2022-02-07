package com.navi.insurance.productFarm.config

import org.springframework.context.annotation.Bean
import org.springframework.context.annotation.Configuration
import springfox.documentation.builders.PathSelectors
import springfox.documentation.builders.RequestHandlerSelectors
import springfox.documentation.service.ApiInfo
import springfox.documentation.service.Contact
import springfox.documentation.spi.DocumentationType
import springfox.documentation.spring.web.plugins.Docket
import java.util.*

@Configuration
class SwaggerConfig {

    @Bean
    fun api(): Docket {
        return Docket(DocumentationType.SWAGGER_2)
                .select()
                .apis(RequestHandlerSelectors.basePackage("com.navi.insurance.productFarm"))
                .paths(PathSelectors.any())
                .build()
                .apiInfo(metaData())
    }

    private fun metaData(): ApiInfo {
        return ApiInfo(
                "Product FARM REST API",
                "This will be used to integrated with all external vendors for general insurance",
                "1.0",
                "Terms of service",
                Contact(
                        "GI Product POD",
                        "https://navihq.atlassian.net/wiki/spaces/NG/pages/384270503/payment-service",
                        "gi-tech@navi.com"
                ),
                "Apache License Version 2.0",
                "https://www.apache.org/licenses/LICENSE-2.0",
                Collections.emptyList()
        )
    }
}

package com.navi.insurance.productFarm.config

import com.navi.insurance.common.interceptor.RequestMetadataHandler
import org.springframework.context.annotation.Configuration
import org.springframework.web.servlet.config.annotation.InterceptorRegistry
import org.springframework.web.servlet.config.annotation.WebMvcConfigurer

@Configuration
class RequestHandlerConfig : WebMvcConfigurer {
    private val requestMetadataHandler = RequestMetadataHandler()
    
    override fun addInterceptors(registry: InterceptorRegistry) {
        registry.addInterceptor(requestMetadataHandler)
    }
}

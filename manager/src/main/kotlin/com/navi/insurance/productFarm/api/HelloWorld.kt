package com.navi.insurance.productFarm.api

import io.sentry.Sentry
import org.apache.logging.log4j.LogManager
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.RestController

@RestController
@RequestMapping("/api")
class HelloWorld {
    
    @GetMapping
    fun findAll() = "Hello World!"
    
    @PostMapping
    fun findAllException() {
        try {
            throw RuntimeException("Hello Exception")
        } catch (e: Exception) {
            Sentry.captureException(e)
        }
        
        try {
            throw RuntimeException("Hello Exception Log")
        } catch (e: Exception) {
            log.error("Exception 2", e)
        }
    }
    
    companion object {
        private val log = LogManager.getLogger()
    }
}

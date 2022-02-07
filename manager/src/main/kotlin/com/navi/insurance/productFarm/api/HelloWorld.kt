package com.navi.insurance.productFarm.api

import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.RestController

@RestController
@RequestMapping("/api")
class HelloWorld {

    @GetMapping
    fun findAll() = "Hello World!"
}
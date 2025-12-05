package io.github.ayushmaanbhav.productFarm.api.attribute

import io.github.ayushmaanbhav.common.model.response.GenericResponse
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.CreateAbstractAttributeRequest
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAbstractAttributeResponse
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PathVariable
import org.springframework.web.bind.annotation.PutMapping
import org.springframework.web.bind.annotation.RequestBody
import org.springframework.web.bind.annotation.RequestMapping

@RequestMapping("/product")
interface AbstractAttributeApi {
    @PutMapping("/{productId}/abstractAttribute")
    fun create(
        @PathVariable productId: String,
        @RequestBody createRequest: CreateAbstractAttributeRequest,
    ): ResponseEntity<GenericResponse<Nothing>>

    @GetMapping("/{productId}/abstractAttribute/{displayName}")
    fun get(
        @PathVariable productId: String,
        @PathVariable displayName: String,
    ): ResponseEntity<GenericResponse<GetAbstractAttributeResponse?>>
}

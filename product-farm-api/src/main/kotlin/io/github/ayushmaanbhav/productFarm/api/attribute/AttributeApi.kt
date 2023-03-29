package io.github.ayushmaanbhav.productFarm.api.attribute

import io.github.ayushmaanbhav.common.model.response.GenericResponse
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.CreateAttributeRequest
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAttributeListByTagResponse
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAttributeResponse
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetFunctionalityAttributeListResponse
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PathVariable
import org.springframework.web.bind.annotation.PutMapping
import org.springframework.web.bind.annotation.RequestBody
import org.springframework.web.bind.annotation.RequestMapping

@RequestMapping("/product")
interface AttributeApi {
    @PutMapping("/{productId}/attribute")
    fun create(
            @PathVariable productId: String,
            @RequestBody createRequest: CreateAttributeRequest,
    ): ResponseEntity<GenericResponse<Nothing>>
    
    @GetMapping("/{productId}/attribute/{displayName}")
    fun get(
        @PathVariable productId: String,
        @PathVariable displayName: String,
    ): ResponseEntity<GenericResponse<GetAttributeResponse?>>
    
    @GetMapping("/{productId}/functionality/{functionality}/attribute")
    fun getFunctionalityAttribute(
        @PathVariable productId: String,
        @PathVariable functionality: String,
    ): ResponseEntity<GenericResponse<GetFunctionalityAttributeListResponse?>>
    
    @GetMapping("/{productId}/attributeByTag/{tag}/attribute")
    fun getAttributeByTag(
        @PathVariable productId: String,
        @PathVariable tag: String,
    ): ResponseEntity<GenericResponse<GetAttributeListByTagResponse?>>
}

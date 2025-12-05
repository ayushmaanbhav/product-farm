package io.github.ayushmaanbhav.productFarm.api.productTemplate

import io.github.ayushmaanbhav.common.model.response.GenericResponse
import io.github.ayushmaanbhav.productFarm.api.productTemplate.dto.ProductTemplateEnumerationDto
import io.github.ayushmaanbhav.productFarm.constant.ProductTemplateType
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PathVariable
import org.springframework.web.bind.annotation.PutMapping
import org.springframework.web.bind.annotation.RequestBody
import org.springframework.web.bind.annotation.RequestMapping

@RequestMapping("/productTemplate")
interface ProductTemplateApi {
    @PutMapping("/{productTemplateType}/enum")
    fun createEnumeration(
        @PathVariable productTemplateType: ProductTemplateType,
        @RequestBody enumerationDto: ProductTemplateEnumerationDto
    ): ResponseEntity<GenericResponse<Nothing>>
    
    @GetMapping("/{productTemplateType}/enum/{name}")
    fun getEnumeration(
        @PathVariable productTemplateType: ProductTemplateType,
        @PathVariable name: String
    ): ResponseEntity<GenericResponse<ProductTemplateEnumerationDto?>>
}

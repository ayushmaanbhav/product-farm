package com.navi.insurance.productFarm.api.v1

import com.navi.insurance.common.model.response.GenericResponse
import com.navi.insurance.productFarm.constant.Constant
import com.navi.insurance.productFarm.constant.ProductTemplateType
import com.navi.insurance.productFarm.dto.ProductTemplateEnumerationDto
import com.navi.insurance.productFarm.service.ProductTemplateService
import org.springframework.http.HttpStatus
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PathVariable
import org.springframework.web.bind.annotation.PutMapping
import org.springframework.web.bind.annotation.RequestMapping
import org.springframework.web.bind.annotation.RestController

@RestController
@RequestMapping("/v1/productTemplate")
class ProductTemplateController(
    val productTemplateService: ProductTemplateService
) {
    @PutMapping("/{productTemplateType}/enum")
    fun createEnumeration(
        @PathVariable productTemplateType: ProductTemplateType, enumerationDto: ProductTemplateEnumerationDto
    ): ResponseEntity<GenericResponse<Nothing>> {
        productTemplateService.createEnumeration(productTemplateType, enumerationDto)
        return GenericResponse.getResponseMessageWithCode(Constant.CREATED_MESSAGE, HttpStatus.CREATED)
    }
    
    @GetMapping("/{productTemplateType}/enum/{name}")
    fun getEnumeration(
        @PathVariable productTemplateType: ProductTemplateType, @PathVariable name: String
    ): ResponseEntity<GenericResponse<ProductTemplateEnumerationDto?>> =
        productTemplateService.getEnumeration(productTemplateType, name).map {
            GenericResponse.getResponseWithCode(it, HttpStatus.OK)
        }.orElseGet {
            GenericResponse.getResponseMessageWithCode(Constant.NOT_FOUND_MESSAGE, HttpStatus.NOT_FOUND)
        }
}

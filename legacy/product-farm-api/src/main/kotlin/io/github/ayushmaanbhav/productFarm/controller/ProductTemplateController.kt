package io.github.ayushmaanbhav.productFarm.controller

import com.github.lkqm.spring.api.version.ApiVersion
import io.github.ayushmaanbhav.common.model.response.GenericResponse
import io.github.ayushmaanbhav.productFarm.api.productTemplate.ProductTemplateApi
import io.github.ayushmaanbhav.productFarm.api.productTemplate.dto.ProductTemplateEnumerationDto
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.constant.ProductTemplateType
import io.github.ayushmaanbhav.productFarm.service.ProductTemplateService
import org.springframework.http.HttpStatus
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.RestController

@ApiVersion("0")
@RestController
class ProductTemplateController(
    private val productTemplateService: ProductTemplateService,
) : ProductTemplateApi {
    override fun createEnumeration(
        productTemplateType: ProductTemplateType, enumerationDto: ProductTemplateEnumerationDto
    ): ResponseEntity<GenericResponse<Nothing>> {
        productTemplateService.createEnumeration(productTemplateType, enumerationDto)
        return GenericResponse.getResponseMessageWithCode(Constant.CREATED_MESSAGE, HttpStatus.CREATED)
    }
    
    override fun getEnumeration(
        productTemplateType: ProductTemplateType, name: String
    ): ResponseEntity<GenericResponse<ProductTemplateEnumerationDto?>> =
        productTemplateService.getEnumeration(productTemplateType, name).map {
            GenericResponse.getResponseWithCode(it, HttpStatus.OK)
        }.orElseGet {
            GenericResponse.getResponseMessageWithCode(Constant.NOT_FOUND_MESSAGE, HttpStatus.NOT_FOUND)
        }
}

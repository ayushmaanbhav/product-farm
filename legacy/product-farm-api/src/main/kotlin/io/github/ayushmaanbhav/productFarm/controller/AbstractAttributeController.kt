package io.github.ayushmaanbhav.productFarm.controller

import com.github.lkqm.spring.api.version.ApiVersion
import io.github.ayushmaanbhav.common.model.response.GenericResponse
import io.github.ayushmaanbhav.productFarm.api.attribute.AbstractAttributeApi
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.CreateAbstractAttributeRequest
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAbstractAttributeResponse
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.service.AbstractAttributeService
import org.springframework.http.HttpStatus
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.RestController

@ApiVersion("0")
@RestController
class AbstractAttributeController(
    private val abstractAttributeService: AbstractAttributeService,
) : AbstractAttributeApi {
    
    override fun create(
        productId: String, createRequest: CreateAbstractAttributeRequest
    ): ResponseEntity<GenericResponse<Nothing>> {
        abstractAttributeService.create(productId, createRequest)
        return GenericResponse.getResponseMessageWithCode(Constant.CREATED_MESSAGE, HttpStatus.CREATED)
    }
    
    override fun get(
        productId: String, displayName: String
    ): ResponseEntity<GenericResponse<GetAbstractAttributeResponse?>> =
        abstractAttributeService.get(productId, displayName).map {
            GenericResponse.getResponseWithCode(it, HttpStatus.OK)
        }.orElseGet {
            GenericResponse.getResponseMessageWithCode(Constant.NOT_FOUND_MESSAGE, HttpStatus.NOT_FOUND)
        }
}

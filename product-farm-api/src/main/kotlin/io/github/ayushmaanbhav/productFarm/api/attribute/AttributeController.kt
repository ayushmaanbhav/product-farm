package io.github.ayushmaanbhav.productFarm.api.attribute

import com.github.lkqm.spring.api.version.ApiVersion
import io.github.ayushmaanbhav.common.model.response.GenericResponse
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.CreateAttributeRequest
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAttributeListByTagResponse
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAttributeResponse
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetFunctionalityAttributeListResponse
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.service.AttributeService
import org.springframework.http.HttpStatus
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.RestController

@ApiVersion("0")
@RestController
class AttributeController(
    private val attributeService: AttributeService,
) : AttributeApi {
    
    override fun create(
        productId: String, createRequest: CreateAttributeRequest
    ): ResponseEntity<GenericResponse<Nothing>> {
        attributeService.create(productId, createRequest)
        return GenericResponse.getResponseMessageWithCode(Constant.CREATED_MESSAGE, HttpStatus.CREATED)
    }
    
    override fun get(productId: String, displayName: String): ResponseEntity<GenericResponse<GetAttributeResponse?>> =
        attributeService.get(productId, displayName).map {
            GenericResponse.getResponseWithCode(it, HttpStatus.OK)
        }.orElseGet {
            GenericResponse.getResponseMessageWithCode(Constant.NOT_FOUND_MESSAGE, HttpStatus.NOT_FOUND)
        }
    
    override fun getFunctionalityAttribute(
        productId: String, functionality: String
    ): ResponseEntity<GenericResponse<GetFunctionalityAttributeListResponse?>> =
        attributeService.getFunctionalityAttribute(productId, functionality).map {
            GenericResponse.getResponseWithCode(it, HttpStatus.OK)
        }.orElseGet {
            GenericResponse.getResponseMessageWithCode(Constant.NOT_FOUND_MESSAGE, HttpStatus.NOT_FOUND)
        }
    
    override fun getAttributeByTag(
        productId: String, tag: String
    ): ResponseEntity<GenericResponse<GetAttributeListByTagResponse?>> =
        attributeService.getAttributeByTag(productId, tag).map {
            GenericResponse.getResponseWithCode(it, HttpStatus.OK)
        }.orElseGet {
            GenericResponse.getResponseMessageWithCode(Constant.NOT_FOUND_MESSAGE, HttpStatus.NOT_FOUND)
        }
}

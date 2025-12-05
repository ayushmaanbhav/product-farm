package io.github.ayushmaanbhav.productFarm.controller

import com.github.lkqm.spring.api.version.ApiVersion
import io.github.ayushmaanbhav.common.model.response.GenericResponse
import io.github.ayushmaanbhav.productFarm.api.productFunctionality.ProductFunctionalityApi
import io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto.CreateProductFunctionalityRequest
import io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto.GetProductFunctionalityResponse
import io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto.ProductFunctionalityApprovalRequest
import io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto.ProductFunctionalityStatusResponse
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.service.ProductFunctionalityService
import org.springframework.http.HttpStatus
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.RestController

@ApiVersion("0")
@RestController
class ProductFunctionalityController(
    private val productFunctionalityService: ProductFunctionalityService,
) : ProductFunctionalityApi {

    override fun create(productId: String, createRequest: CreateProductFunctionalityRequest): ResponseEntity<GenericResponse<Nothing>> {
        productFunctionalityService.create(productId, createRequest)
        return GenericResponse.getResponseMessageWithCode(Constant.CREATED_MESSAGE, HttpStatus.CREATED)
    }

    override fun get(productId: String, name: String): ResponseEntity<GenericResponse<GetProductFunctionalityResponse?>> =
        productFunctionalityService.get(productId, name).map {
            GenericResponse.getResponseWithCode(it, HttpStatus.OK)
        }.orElseGet {
            GenericResponse.getResponseMessageWithCode(Constant.NOT_FOUND_MESSAGE, HttpStatus.NOT_FOUND)
        }

    override fun submit(productId: String, name: String): ResponseEntity<GenericResponse<ProductFunctionalityStatusResponse?>> =
        productFunctionalityService.submit(productId, name).map {
            GenericResponse.getResponseWithCode(it, HttpStatus.OK)
        }.orElseGet {
            GenericResponse.getResponseMessageWithCode(Constant.NOT_FOUND_MESSAGE, HttpStatus.NOT_FOUND)
        }

    override fun approve(
        productId: String, name: String, approvalRequest: ProductFunctionalityApprovalRequest
    ): ResponseEntity<GenericResponse<ProductFunctionalityStatusResponse?>> =
        productFunctionalityService.approve(productId, name).map {
            GenericResponse.getResponseWithCode(it, HttpStatus.OK)
        }.orElseGet {
            GenericResponse.getResponseMessageWithCode(Constant.NOT_FOUND_MESSAGE, HttpStatus.NOT_FOUND)
        }
}

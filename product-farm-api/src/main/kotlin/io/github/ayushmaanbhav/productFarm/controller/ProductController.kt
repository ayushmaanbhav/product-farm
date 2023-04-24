package io.github.ayushmaanbhav.productFarm.controller

import com.github.lkqm.spring.api.version.ApiVersion
import io.github.ayushmaanbhav.common.model.response.GenericResponse
import io.github.ayushmaanbhav.productFarm.api.product.ProductApi
import io.github.ayushmaanbhav.productFarm.api.product.dto.CloneProductRequest
import io.github.ayushmaanbhav.productFarm.api.product.dto.CreateProductRequest
import io.github.ayushmaanbhav.productFarm.api.product.dto.GetProductResponse
import io.github.ayushmaanbhav.productFarm.api.product.dto.ProductApprovalRequest
import io.github.ayushmaanbhav.productFarm.api.product.dto.ProductApprovalResponse
import io.github.ayushmaanbhav.productFarm.api.product.dto.SubmitProductResponse
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.service.CloneProductService
import io.github.ayushmaanbhav.productFarm.service.ProductService
import org.springframework.http.HttpStatus
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.RestController

@ApiVersion("0")
@RestController
class ProductController(
    private val productService: ProductService,
    private val cloneProductService: CloneProductService,
) : ProductApi {
    
    override fun create(createRequest: CreateProductRequest): ResponseEntity<GenericResponse<Nothing>> {
        productService.create(createRequest)
        return GenericResponse.getResponseMessageWithCode(Constant.CREATED_MESSAGE, HttpStatus.CREATED)
    }
    
    override fun get(productId: String): ResponseEntity<GenericResponse<GetProductResponse?>> =
        productService.getProduct(productId).map {
            GenericResponse.getResponseWithCode(it, HttpStatus.OK)
        }.orElseGet {
            GenericResponse.getResponseMessageWithCode(Constant.NOT_FOUND_MESSAGE, HttpStatus.NOT_FOUND)
        }
    
    override fun clone(
        parentProductId: String, cloneRequest: CloneProductRequest
    ): ResponseEntity<GenericResponse<Nothing>> {
        cloneProductService.clone(parentProductId, cloneRequest)
        return GenericResponse.getResponseMessageWithCode(Constant.CREATED_MESSAGE, HttpStatus.CREATED)
    }
    
    override fun submit(productId: String): ResponseEntity<GenericResponse<SubmitProductResponse?>> =
        productService.submit(productId).let {
            GenericResponse.getResponseWithCode(it, HttpStatus.OK)
        }
    
    override fun approve(
        productId: String, approvalRequest: ProductApprovalRequest
    ): ResponseEntity<GenericResponse<ProductApprovalResponse?>> =
        productService.approve(productId, approvalRequest).let {
            GenericResponse.getResponseWithCode(it, HttpStatus.OK)
        }
}

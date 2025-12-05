package io.github.ayushmaanbhav.productFarm.api.productFunctionality

import io.github.ayushmaanbhav.common.model.response.GenericResponse
import io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto.CreateProductFunctionalityRequest
import io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto.GetProductFunctionalityResponse
import io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto.ProductFunctionalityApprovalRequest
import io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto.ProductFunctionalityStatusResponse
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PathVariable
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.PutMapping
import org.springframework.web.bind.annotation.RequestBody
import org.springframework.web.bind.annotation.RequestMapping

@RequestMapping("/product")
interface ProductFunctionalityApi {
    @PutMapping("/{productId}/functionality")
    fun create(
        @PathVariable productId: String,
        @RequestBody createRequest: CreateProductFunctionalityRequest
    ): ResponseEntity<GenericResponse<Nothing>>
    
    @GetMapping("/{productId}/functionality/{name}")
    fun get(
        @PathVariable productId: String,
        @PathVariable name: String
    ): ResponseEntity<GenericResponse<GetProductFunctionalityResponse?>>
    
    @PostMapping("/{productId}/functionality/{name}/submit")
    fun submit(
        @PathVariable productId: String,
        @PathVariable name: String
    ): ResponseEntity<GenericResponse<ProductFunctionalityStatusResponse?>>
    
    @PostMapping("/{productId}/functionality/{name}/approve")
    fun approve(
        @PathVariable productId: String,
        @PathVariable name: String,
        @RequestBody approvalRequest: ProductFunctionalityApprovalRequest,
    ): ResponseEntity<GenericResponse<ProductFunctionalityStatusResponse?>>
}

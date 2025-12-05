package io.github.ayushmaanbhav.productFarm.api.product

import io.github.ayushmaanbhav.common.model.response.GenericResponse
import io.github.ayushmaanbhav.productFarm.api.product.dto.CloneProductRequest
import io.github.ayushmaanbhav.productFarm.api.product.dto.CreateProductRequest
import io.github.ayushmaanbhav.productFarm.api.product.dto.GetProductResponse
import io.github.ayushmaanbhav.productFarm.api.product.dto.ProductApprovalRequest
import io.github.ayushmaanbhav.productFarm.api.product.dto.ProductApprovalResponse
import io.github.ayushmaanbhav.productFarm.api.product.dto.SubmitProductResponse
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.GetMapping
import org.springframework.web.bind.annotation.PathVariable
import org.springframework.web.bind.annotation.PostMapping
import org.springframework.web.bind.annotation.PutMapping
import org.springframework.web.bind.annotation.RequestBody
import org.springframework.web.bind.annotation.RequestMapping

@RequestMapping("/product")
interface ProductApi {
    @PutMapping
    fun create(@RequestBody createRequest: CreateProductRequest): ResponseEntity<GenericResponse<Nothing>>
    
    @GetMapping("/{productId}")
    fun get(@PathVariable productId: String): ResponseEntity<GenericResponse<GetProductResponse?>>
    
    @PutMapping("/{parentProductId}/clone")
    fun clone(
        @PathVariable parentProductId: String,
        @RequestBody cloneRequest: CloneProductRequest
    ): ResponseEntity<GenericResponse<Nothing>>
    
    @PostMapping("/{productId}/submit")
    fun submit(@PathVariable productId: String): ResponseEntity<GenericResponse<SubmitProductResponse?>>
    
    @PostMapping("/{productId}/approve")
    fun approve(
        @PathVariable productId: String,
        @RequestBody approvalRequest: ProductApprovalRequest
    ): ResponseEntity<GenericResponse<ProductApprovalResponse?>>
}

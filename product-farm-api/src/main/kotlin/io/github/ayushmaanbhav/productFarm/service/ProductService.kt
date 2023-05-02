package io.github.ayushmaanbhav.productFarm.service

import io.github.ayushmaanbhav.common.exception.ValidatorException
import io.github.ayushmaanbhav.productFarm.api.product.dto.CloneProductRequest
import io.github.ayushmaanbhav.productFarm.api.product.dto.CreateProductRequest
import io.github.ayushmaanbhav.productFarm.api.product.dto.GetProductResponse
import io.github.ayushmaanbhav.productFarm.api.product.dto.ProductApprovalRequest
import io.github.ayushmaanbhav.productFarm.api.product.dto.ProductApprovalResponse
import io.github.ayushmaanbhav.productFarm.api.product.dto.SubmitProductResponse
import io.github.ayushmaanbhav.productFarm.constant.ProductStatus
import io.github.ayushmaanbhav.productFarm.entity.repository.ProductRepo
import io.github.ayushmaanbhav.productFarm.transformer.CreateProductTransformer
import io.github.ayushmaanbhav.productFarm.transformer.GetProductTransformer
import io.github.ayushmaanbhav.productFarm.util.createError
import jakarta.transaction.Transactional
import java.util.*
import org.springframework.http.HttpStatus
import org.springframework.stereotype.Component

@Component
class ProductService(
    private val createProductTransformer: CreateProductTransformer,
    private val getProductTransformer: GetProductTransformer,
    private val productRepo: ProductRepo,
    private val productApprovalService: ProductApprovalService,
) {
    @Transactional
    fun create(request: CreateProductRequest) {
        if (productRepo.existsById(request.id)) {
            throw ValidatorException(
                HttpStatus.BAD_REQUEST.value(), listOf(createError("Product already exists for this id"))
            )
        }
        productRepo.save(createProductTransformer.forward(request))
    }
    
    fun getProduct(id: String): Optional<GetProductResponse> {
        return productRepo.findById(id).map { getProductTransformer.forward(it) }
    }
    
    @Transactional
    fun submit(id: String): SubmitProductResponse {
        if (productRepo.existsById(id).not()) {
            throw ValidatorException(
                HttpStatus.NOT_FOUND.value(), listOf(createError("Product does not exist for this id"))
            )
        }
        val product = productRepo.getReferenceById(id)
        if (product.status != ProductStatus.DRAFT) {
            throw ValidatorException(
                HttpStatus.BAD_REQUEST.value(), listOf(createError("Product is not in draft status"))
            )
        }
        product.status = ProductStatus.PENDING_APPROVAL
        productRepo.save(product)
        return SubmitProductResponse(product.status)
    }
    
    @Transactional
    fun approve(id: String, approvalRequest: ProductApprovalRequest): ProductApprovalResponse {
        if (productRepo.existsById(id).not()) {
            throw ValidatorException(
                HttpStatus.NOT_FOUND.value(), listOf(createError("Product does not exist for this id"))
            )
        }
        val product = productRepo.getReferenceById(id)
        if (product.status != ProductStatus.PENDING_APPROVAL) {
            throw ValidatorException(
                HttpStatus.BAD_REQUEST.value(), listOf(createError("Product is not in pending approval status"))
            )
        }
        approvalRequest.discontinuedProductId?.let {
            if (productRepo.existsById(it).not()) {
                throw ValidatorException(
                    HttpStatus.NOT_FOUND.value(), listOf(createError("Product to discontinue does not exist for id"))
                )
            }
            val productToDiscontinue = productRepo.getReferenceById(it)
            if (productToDiscontinue.status != ProductStatus.ACTIVE) {
                throw ValidatorException(
                    HttpStatus.BAD_REQUEST.value(), listOf(createError("Product to discontinue not in active status"))
                )
            }
            productToDiscontinue.status = ProductStatus.DISCONTINUED
            productRepo.save(productToDiscontinue)
        }
        product.status = ProductStatus.ACTIVE
        productRepo.save(product)
        productApprovalService.create(id, approvalRequest)
        return ProductApprovalResponse(
            status = product.status,
            effectiveFrom = product.effectiveFrom,
            expiryAt = product.expiryAt,
            approvedBy = approvalRequest.approvedBy,
            discontinuedProductId = approvalRequest.discontinuedProductId,
            changeDescription = approvalRequest.changeDescription,
        )
    }
    
    @Transactional
    fun clone(parentProductId: String, request: CloneProductRequest) {
        if (productRepo.existsById(request.productId)) {
            throw ValidatorException(
                HttpStatus.BAD_REQUEST.value(), listOf(createError("Product already exists for this id"))
            )
        }
        if (productRepo.existsById(parentProductId).not()) {
            throw ValidatorException(
                HttpStatus.NOT_FOUND.value(), listOf(createError("Product does not exist for this id"))
            )
        }
        val parentProduct = productRepo.getReferenceById(parentProductId)
        val createProductRequest = CreateProductRequest(
            id = request.productId,
            name = request.name,
            effectiveFrom = request.effectiveFrom,
            expiryAt = request.expiryAt,
            templateType = parentProduct.templateType,
            description = request.description,
        )
        create(createProductRequest)
    }
}

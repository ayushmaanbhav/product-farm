package io.github.ayushmaanbhav.productFarm.service

import io.github.ayushmaanbhav.common.exception.ValidatorException
import io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto.CreateProductFunctionalityRequest
import io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto.GetProductFunctionalityResponse
import io.github.ayushmaanbhav.productFarm.api.productFunctionality.dto.ProductFunctionalityStatusResponse
import io.github.ayushmaanbhav.productFarm.constant.ProductFunctionalityStatus
import io.github.ayushmaanbhav.productFarm.entity.repository.AbstractAttributeRepo
import io.github.ayushmaanbhav.productFarm.entity.repository.ProductFunctionalityRepo
import io.github.ayushmaanbhav.productFarm.transformer.CreateProductFunctionalityTransformer
import io.github.ayushmaanbhav.productFarm.transformer.GetProductFunctionalityStatusTransformer
import io.github.ayushmaanbhav.productFarm.transformer.GetProductFunctionalityTransformer
import io.github.ayushmaanbhav.productFarm.util.createError
import jakarta.transaction.Transactional
import java.util.*
import org.springframework.http.HttpStatus
import org.springframework.stereotype.Component

@Component
class ProductFunctionalityService(
    val abstractAttributeRepo: AbstractAttributeRepo,
    val productFunctionalityRepo: ProductFunctionalityRepo,
    val createProductFunctionalityTransformer: CreateProductFunctionalityTransformer,
    val getProductFunctionalityTransformer: GetProductFunctionalityTransformer,
    val getProductFunctionalityStatusTransformer: GetProductFunctionalityStatusTransformer,
) {
    @Transactional
    fun create(productId: String, request: CreateProductFunctionalityRequest) {
        if (productFunctionalityRepo.existsByProductIdAndName(productId, request.name)) {
            throw ValidatorException(HttpStatus.BAD_REQUEST.value(), listOf(createError("Functionality already exists for this id")))
        }
        productFunctionalityRepo.save(createProductFunctionalityTransformer.forward(Pair(productId, request)))
    }

    fun get(productId: String, name: String): Optional<GetProductFunctionalityResponse> =
        productFunctionalityRepo.findByProductIdAndName(productId, name).map { getProductFunctionalityTransformer.forward(it) }

    @Transactional
    fun submit(productId: String, name: String): Optional<ProductFunctionalityStatusResponse> =
        productFunctionalityRepo.findByProductIdAndName(productId, name).map {
            if (it.status != ProductFunctionalityStatus.DRAFT) {
                throw ValidatorException(HttpStatus.BAD_REQUEST.value(), listOf(createError("Invalid request, not in draft status")))
            }
            if (it.requiredAttributes.isEmpty()) {
                throw ValidatorException(HttpStatus.BAD_REQUEST.value(), listOf(createError("No attributes found in functionality")))
            }
            it.status = ProductFunctionalityStatus.PENDING_APPROVAL
            productFunctionalityRepo.save(it)
            getProductFunctionalityStatusTransformer.forward(it)
        }

    @Transactional
    fun approve(productId: String, name: String): Optional<ProductFunctionalityStatusResponse> =
        productFunctionalityRepo.findByProductIdAndName(productId, name).map {
            if (it.status != ProductFunctionalityStatus.PENDING_APPROVAL) {
                throw ValidatorException(HttpStatus.BAD_REQUEST.value(), listOf(createError("Not in pending approval status")))
            }
            if (it.immutable) {
                it.requiredAttributes.map { it1 -> abstractAttributeRepo.getReferenceById(it1.id.abstractPath) }
                    .filterNot { it1 -> it1.immutable }
                    .map { it1 ->
                        it1.immutable = true
                        abstractAttributeRepo.save(it1)
                    }
            }
            it.status = ProductFunctionalityStatus.ACTIVE
            productFunctionalityRepo.save(it)
            getProductFunctionalityStatusTransformer.forward(it)
        }

    fun clone(parentProductId: String, productId: String) {
        TODO()
    }
}

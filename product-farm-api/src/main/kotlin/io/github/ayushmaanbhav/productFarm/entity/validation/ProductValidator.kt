package io.github.ayushmaanbhav.productFarm.entity.validation

import ValidProduct
import io.github.ayushmaanbhav.common.model.response.ErrorDetail
import io.github.ayushmaanbhav.common.validator.exception.ValidatorException
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.constant.ProductStatus.DISCONTINUED
import io.github.ayushmaanbhav.productFarm.entity.Product
import io.github.ayushmaanbhav.productFarm.entity.repository.ProductApprovalRepo
import io.github.ayushmaanbhav.productFarm.exception.ProductFarmServiceException
import io.github.ayushmaanbhav.productFarm.validation.createError
import io.github.ayushmaanbhav.productFarm.validation.populateProperty
import org.apache.logging.log4j.LogManager
import org.springframework.http.HttpStatus
import org.springframework.stereotype.Component
import java.time.LocalDateTime
import jakarta.validation.ConstraintValidator
import jakarta.validation.ConstraintValidatorContext
import kotlin.reflect.full.memberProperties

@Component
class ProductValidator(
    private val productApprovalRepo: ProductApprovalRepo,
) : ConstraintValidator<ValidProduct, Product> {
    
    override fun isValid(product: Product, cxt: ConstraintValidatorContext): Boolean {
        val approved = productApprovalRepo.existsById(product.id)
        val errorList = mutableListOf<ErrorDetail>()
        for (property in Product::class.memberProperties) {
            val errorDetail: ErrorDetail? = when (property) {
                Product::description ->
                    createError()
                        .takeUnless { product.description?.let(Constant.DESCRIPTION_REGEX::matches) ?: true }
                Product::effectiveFrom ->
                    createError()
                        .takeUnless { product.effectiveFrom.isAfter(LocalDateTime.now()) }
                Product::expiryAt ->
                    createError()
                        .takeUnless { product.expiryAt.isAfter(product.effectiveFrom) }
                Product::id ->
                    createError()
                        .takeUnless { product.id.let(Constant.PRODUCT_ID_REGEX::matches) }
                Product::name ->
                    createError()
                        .takeUnless { product.name.let(Constant.PRODUCT_NAME_REGEX::matches) }
                Product::parentProductId -> null // fk
                Product::status ->
                    createError()
                        .takeUnless {
                            approved.not()
                            || (product.status == DISCONTINUED && LocalDateTime.now().isAfter(product.expiryAt))
                        }
                Product::templateType -> null // enum
                else -> throw ProductFarmServiceException(
                    "Missing validation for property", arrayOf(property.name, property.javaClass.name)
                )
            }
            errorDetail?.let { errorList.add(populateProperty(it, property)) }
        }
        if (errorList.isNotEmpty()) {
            log.info("Error: ", errorList)
            throw ValidatorException(HttpStatus.BAD_REQUEST.value(), errorList)
        }
        return true
    }
    
    companion object {
        private val log = LogManager.getLogger()
    }
}

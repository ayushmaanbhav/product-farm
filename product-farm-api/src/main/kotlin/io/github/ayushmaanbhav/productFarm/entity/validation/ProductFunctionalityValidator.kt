package io.github.ayushmaanbhav.productFarm.entity.validation

import ValidProductFunctionality
import io.github.ayushmaanbhav.common.model.response.ErrorDetail
import io.github.ayushmaanbhav.common.exception.ValidatorException
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.constant.ProductFunctionalityStatus.ACTIVE
import io.github.ayushmaanbhav.productFarm.constant.ProductStatus.DRAFT
import io.github.ayushmaanbhav.productFarm.constant.ProductStatus.PENDING_APPROVAL
import io.github.ayushmaanbhav.productFarm.entity.ProductFunctionality
import io.github.ayushmaanbhav.productFarm.entity.repository.ProductRepo
import io.github.ayushmaanbhav.productFarm.exception.ProductFarmServiceException
import io.github.ayushmaanbhav.productFarm.util.createError
import io.github.ayushmaanbhav.productFarm.util.populateProperty
import org.apache.logging.log4j.LogManager
import org.springframework.http.HttpStatus
import org.springframework.stereotype.Component
import jakarta.validation.ConstraintValidator
import jakarta.validation.ConstraintValidatorContext
import kotlin.reflect.full.memberProperties

@Component
class ProductFunctionalityValidator(
    private val productRepo: ProductRepo,
) : ConstraintValidator<ValidProductFunctionality, ProductFunctionality> {
    
    override fun isValid(productFunctionality: ProductFunctionality, cxt: ConstraintValidatorContext): Boolean {
        val product = productRepo.getReferenceById(productFunctionality.productId)
        val errorList = mutableListOf<ErrorDetail>()
        for (property in ProductFunctionality::class.memberProperties) {
            val errorDetail: ErrorDetail? = when (property) {
                ProductFunctionality::description ->
                    createError()
                        .takeUnless { productFunctionality.description.let(Constant.DESCRIPTION_REGEX::matches) }
                ProductFunctionality::id ->
                    createError()
                        .takeUnless { productFunctionality.id.let(Constant.UUID_REGEX::matches) }
                ProductFunctionality::immutable -> null
                ProductFunctionality::name ->
                    createError()
                        .takeUnless { productFunctionality.name.let(Constant.FUNCTIONALITY_NAME_REGEX::matches) }
                ProductFunctionality::productId -> null // fk
                ProductFunctionality::requiredAttributes ->
                    createError()
                        .takeUnless {
                            productFunctionality.requiredAttributes
                                .mapIndexed { index, tag -> tag.order == index }
                                .all { it }
                        }
                ProductFunctionality::status ->
                    createError()
                        .takeIf {
                            product.status in setOf(DRAFT, PENDING_APPROVAL) && productFunctionality.status == ACTIVE
                        }
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

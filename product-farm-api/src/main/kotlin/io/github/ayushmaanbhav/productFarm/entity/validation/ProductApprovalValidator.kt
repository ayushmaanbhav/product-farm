package io.github.ayushmaanbhav.productFarm.entity.validation

import ValidProductApproval
import io.github.ayushmaanbhav.common.model.response.ErrorDetail
import io.github.ayushmaanbhav.common.exception.ValidatorException
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.constant.ProductStatus.ACTIVE
import io.github.ayushmaanbhav.productFarm.constant.ProductStatus.DISCONTINUED
import io.github.ayushmaanbhav.productFarm.entity.ProductApproval
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
class ProductApprovalValidator(
    private val productRepo: ProductRepo,
) : ConstraintValidator<ValidProductApproval, ProductApproval> {
    
    override fun isValid(productApproval: ProductApproval, cxt: ConstraintValidatorContext): Boolean {
        val errorList = mutableListOf<ErrorDetail>()
        for (property in ProductApproval::class.memberProperties) {
            val errorDetail: ErrorDetail? = when (property) {
                ProductApproval::approvedBy ->
                    createError()
                        .takeUnless { productApproval.approvedBy.let(Constant.DESCRIPTION_REGEX::matches) }
                ProductApproval::changeDescription ->
                    createError()
                        .takeUnless { productApproval.changeDescription.let(Constant.DESCRIPTION_REGEX::matches) }
                ProductApproval::discontinuedProductId ->
                    createError()
                        .takeUnless {
                            productApproval.discontinuedProductId?.let { id ->
                                val discontinuedProduct = productRepo.getReferenceById(id)
                                discontinuedProduct.status == DISCONTINUED
                            } ?: true
                        }
                ProductApproval::productId ->
                    createError()
                        .takeUnless {
                            val approvedProduct = productRepo.getReferenceById(productApproval.productId)
                            approvedProduct.status == ACTIVE
                        }
                else -> throw ProductFarmServiceException(
                    "Missing validation for property", arrayOf(property.name, property.javaClass.name)
                )
            }
            errorDetail?.let { errorList.add(populateProperty(it, property)) }
        }
        if (errorList.isNotEmpty()) {
            log.info("Error: $errorList")
            throw ValidatorException(HttpStatus.BAD_REQUEST.value(), errorList)
        }
        return true
    }
    
    companion object {
        private val log = LogManager.getLogger()
    }
}

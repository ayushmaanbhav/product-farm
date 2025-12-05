package io.github.ayushmaanbhav.productFarm.entity.validation

import ValidProductTemplateEnumeration
import io.github.ayushmaanbhav.common.exception.ValidatorException
import io.github.ayushmaanbhav.common.model.response.ErrorDetail
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.entity.ProductTemplateEnumeration
import io.github.ayushmaanbhav.productFarm.exception.ProductFarmServiceException
import io.github.ayushmaanbhav.productFarm.util.createError
import io.github.ayushmaanbhav.productFarm.util.populateProperty
import jakarta.validation.ConstraintValidator
import jakarta.validation.ConstraintValidatorContext
import org.apache.logging.log4j.LogManager
import org.springframework.http.HttpStatus
import org.springframework.stereotype.Component
import kotlin.reflect.full.declaredMemberProperties

@Component
class ProductTemplateEnumerationValidator
    : ConstraintValidator<ValidProductTemplateEnumeration, ProductTemplateEnumeration> {
    
    override fun isValid(enumeration: ProductTemplateEnumeration, cxt: ConstraintValidatorContext): Boolean {
        val errorList = mutableListOf<ErrorDetail>()
        for (property in ProductTemplateEnumeration::class.declaredMemberProperties) {
            val errorDetail: ErrorDetail? = when (property) {
                ProductTemplateEnumeration::description ->
                    createError()
                        .takeUnless { enumeration.description?.let(Constant.DESCRIPTION_REGEX::matches) ?: true }
                ProductTemplateEnumeration::id ->
                    createError()
                        .takeUnless { enumeration.id.let(Constant.UUID_REGEX::matches) }
                ProductTemplateEnumeration::name ->
                    createError()
                        .takeUnless { enumeration.name.let(Constant.ENUMERATION_NAME_REGEX::matches) }
                ProductTemplateEnumeration::productTemplateType -> null // enum
                ProductTemplateEnumeration::values ->
                    createError()
                        .takeUnless { enumeration.values.all(Constant.ENUMERATION_VALUE_REGEX::matches) }
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

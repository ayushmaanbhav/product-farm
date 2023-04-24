package io.github.ayushmaanbhav.productFarm.entity.validation

import ValidDatatype
import io.github.ayushmaanbhav.common.model.response.ErrorDetail
import io.github.ayushmaanbhav.common.exception.ValidatorException
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.entity.Datatype
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
class DatatypeValidator : ConstraintValidator<ValidDatatype, Datatype> {
    
    override fun isValid(datatype: Datatype, cxt: ConstraintValidatorContext): Boolean {
        val errorList = mutableListOf<ErrorDetail>()
        for (property in Datatype::class.memberProperties) {
            val errorDetail: ErrorDetail? = when (property) {
                Datatype::description ->
                    createError()
                        .takeUnless { datatype.description?.let(Constant.DESCRIPTION_REGEX::matches) ?: true }
                Datatype::name ->
                    createError()
                        .takeUnless { datatype.name.let(Constant.DATATYPE_REGEX::matches) }
                Datatype::type -> null // enum
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

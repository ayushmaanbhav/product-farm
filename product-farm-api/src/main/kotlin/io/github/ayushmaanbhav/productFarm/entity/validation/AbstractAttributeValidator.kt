package io.github.ayushmaanbhav.productFarm.entity.validation

import ValidAbstractAttribute
import io.github.ayushmaanbhav.common.model.response.ErrorDetail
import io.github.ayushmaanbhav.common.exception.ValidatorException
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.constant.ProductStatus.DRAFT
import io.github.ayushmaanbhav.productFarm.entity.AbstractAttribute
import io.github.ayushmaanbhav.productFarm.entity.repository.AbstractAttributeRepo
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
class AbstractAttributeValidator(
    private val productRepo: ProductRepo,
    private val abstractAttributeRepo: AbstractAttributeRepo,
    private val datatypeValidator: DatatypeValidator,
    private val enumerationValidator: ProductTemplateEnumerationValidator,
    private val ruleValidator: RuleValidator,
) : ConstraintValidator<ValidAbstractAttribute, AbstractAttribute> {
    
    override fun isValid(abstractAttribute: AbstractAttribute, cxt: ConstraintValidatorContext): Boolean {
        val errorList = mutableListOf<ErrorDetail>()
        for (property in AbstractAttribute::class.memberProperties) {
            val errorDetail: ErrorDetail? = when (property) {
                AbstractAttribute::abstractPath ->
                    createError()
                        .takeUnless { abstractAttribute.abstractPath.let(Constant.ABSTRACT_PATH_REGEX::matches) }
                AbstractAttribute::componentId ->
                    createError()
                        .takeUnless {
                            abstractAttribute.componentId?.let(Constant.COMPONENT_ID_REGEX::matches) ?: true
                        }
                AbstractAttribute::componentType ->
                    createError()
                        .takeUnless { abstractAttribute.componentType.let(Constant.COMPONENT_TYPE_REGEX::matches) }
                AbstractAttribute::constraintRule ->
                    createError()
                        .takeUnless {
                            abstractAttribute.constraintRule?.let { rule -> ruleValidator.isValid(rule, cxt) } ?: true
                        }
                AbstractAttribute::datatype ->
                    createError()
                        .takeUnless { datatypeValidator.isValid(abstractAttribute.datatype, cxt) }
                AbstractAttribute::description ->
                    createError()
                        .takeUnless { abstractAttribute.description?.let(Constant.DESCRIPTION_REGEX::matches) ?: true }
                AbstractAttribute::displayNames ->
                    createError()
                        .takeUnless {
                            abstractAttribute.displayNames.all {
                                it.id.displayName.let(Constant.DISPLAY_NAME_REGEX::matches)
                            }
                        }
                AbstractAttribute::enumeration ->
                    createError()
                        .takeUnless {
                            abstractAttribute.enumeration?.let { it1 ->
                                enumerationValidator.isValid(it1, cxt)
                            } ?: true
                        }
                AbstractAttribute::immutable ->
                    createError()
                        .takeUnless {
                            abstractAttribute.immutable
                            && (productRepo.getReferenceById(abstractAttribute.productId).status == DRAFT
                                || abstractAttributeRepo.existsById(abstractAttribute.abstractPath).not())
                        }
                AbstractAttribute::productId -> null // fk
                AbstractAttribute::relatedAttributes ->
                    createError()
                        .takeUnless {
                            abstractAttribute.relatedAttributes
                                .mapIndexed { index, relatedAttribute ->
                                    relatedAttribute.order == index
                                    && relatedAttribute.id.relationship.let(Constant.RELATIONSHIP_NAME_REGEX::matches)
                                }
                                .all { it }
                        }
                AbstractAttribute::tags ->
                    createError()
                        .takeUnless { abstractAttribute.componentType.let(Constant.TAG_REGEX::matches) }
                        .takeUnless {
                            abstractAttribute.tags
                                .mapIndexed { index, tag -> tag.order == index }
                                .all { it }
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

package io.github.ayushmaanbhav.productFarm.entity.validation

import ValidRule
import io.github.ayushmaanbhav.common.model.response.ErrorDetail
import io.github.ayushmaanbhav.common.validator.exception.ValidatorException
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.entity.Rule
import io.github.ayushmaanbhav.productFarm.exception.ProductFarmServiceException
import io.github.ayushmaanbhav.productFarm.transformer.RuleTransformer
import io.github.ayushmaanbhav.productFarm.util.RuleUtil
import io.github.ayushmaanbhav.productFarm.validation.createError
import io.github.ayushmaanbhav.productFarm.validation.populateProperty
import org.apache.logging.log4j.LogManager
import org.springframework.http.HttpStatus
import org.springframework.stereotype.Component
import jakarta.validation.ConstraintValidator
import jakarta.validation.ConstraintValidatorContext
import kotlin.reflect.full.memberProperties

@Component
class RuleValidator(
    private val ruleTransformer: RuleTransformer,
    private val ruleUtil: RuleUtil,
) : ConstraintValidator<ValidRule, Rule> {
    
    override fun isValid(rule: Rule, cxt: ConstraintValidatorContext): Boolean {
        val errorList = mutableListOf<ErrorDetail>()
        for (property in Rule::class.memberProperties) {
            val errorDetail: ErrorDetail? = when (property) {
                Rule::compiledExpression ->
                    createError()
                        .takeUnless {
                            rule.compiledExpression == ruleUtil.compileExpression(ruleTransformer.forward(rule))
                        }
                Rule::id ->
                    createError()
                        .takeUnless { Constant.UUID_REGEX.matches(rule.id) }
                Rule::inputAttributes ->
                    createError()
                        .takeUnless {
                            rule.inputAttributes
                                .mapIndexed { index, ruleInputAttribute -> ruleInputAttribute.order == index }
                                .all { it }
                        }
                Rule::outputAttributes ->
                    createError()
                        .takeUnless {
                            rule.outputAttributes
                                .mapIndexed { index, ruleOutputAttribute -> ruleOutputAttribute.order == index }
                                .all { it }
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

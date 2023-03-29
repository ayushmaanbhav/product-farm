package io.github.ayushmaanbhav.productFarm.entity.validation

import ValidAttributeDirectedAcyclicGraph
import io.github.ayushmaanbhav.common.validator.exception.ValidatorException
import io.github.ayushmaanbhav.productFarm.entity.Attribute
import io.github.ayushmaanbhav.productFarm.entity.Rule
import io.github.ayushmaanbhav.productFarm.entity.repository.AttributeRepo
import io.github.ayushmaanbhav.productFarm.util.RuleUtil
import io.github.ayushmaanbhav.productFarm.validation.createError
import io.github.ayushmaanbhav.rule.domain.ruleEngine.generic.exception.GraphContainsCycleException
import org.apache.logging.log4j.LogManager
import org.springframework.http.HttpStatus.BAD_REQUEST
import org.springframework.stereotype.Component
import jakarta.validation.ConstraintValidator
import jakarta.validation.ConstraintValidatorContext

@Component
class AttributeDirectedAcyclicGraphValidator(
    private val attributeRepo: AttributeRepo,
    private val ruleUtil: RuleUtil,
) : ConstraintValidator<ValidAttributeDirectedAcyclicGraph, Attribute> {
    
    override fun isValid(attribute: Attribute, cxt: ConstraintValidatorContext): Boolean {
        val existingAttribute = attributeRepo.findAllByProductIdOrderByPathAsc(attribute.productId)
        val allAttribute = LinkedHashSet<Attribute>(existingAttribute)
        allAttribute.add(attribute)
        val ruleList = LinkedHashSet<Rule>()
        allAttribute.forEach { it.rule?.let(ruleList::add) }
        try {
            ruleUtil.createRuleDependencyGraph(ruleList)
        } catch (error: GraphContainsCycleException) {
            log.info("Error: ", error)
            throw ValidatorException(BAD_REQUEST.value(), listOf(createError("rule dependency api contains cycle")))
        }
        return true
    }
    
    companion object {
        private val log = LogManager.getLogger()
    }
}
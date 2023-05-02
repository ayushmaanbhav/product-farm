package io.github.ayushmaanbhav.productFarm.entity.validation

import ValidAttributeDirectedAcyclicGraph
import io.github.ayushmaanbhav.common.exception.ValidatorException
import io.github.ayushmaanbhav.productFarm.entity.Attribute
import io.github.ayushmaanbhav.productFarm.entity.repository.AttributeRepo
import io.github.ayushmaanbhav.productFarm.model.Rule
import io.github.ayushmaanbhav.productFarm.transformer.RuleTransformer
import io.github.ayushmaanbhav.productFarm.util.RuleUtil
import io.github.ayushmaanbhav.productFarm.util.createError
import io.github.ayushmaanbhav.ruleEngine.exception.GraphContainsCycleException
import jakarta.validation.ConstraintValidator
import jakarta.validation.ConstraintValidatorContext
import org.apache.logging.log4j.LogManager
import org.springframework.http.HttpStatus.BAD_REQUEST
import org.springframework.stereotype.Component

@Component
class AttributeDirectedAcyclicGraphValidator(
    private val attributeRepo: AttributeRepo,
    private val ruleUtil: RuleUtil,
    private val ruleTransformer: RuleTransformer,
) : ConstraintValidator<ValidAttributeDirectedAcyclicGraph, Attribute> {
    
    override fun isValid(attribute: Attribute, cxt: ConstraintValidatorContext): Boolean {
        val existingAttribute = attributeRepo.findAllByProductIdOrderByPathAsc(attribute.productId)
        val allAttribute = LinkedHashSet<Attribute>(existingAttribute)
        allAttribute.add(attribute)
        val ruleList = LinkedHashSet<Rule>()
        allAttribute.forEach { it.rule?.let(ruleTransformer::forward)?.let(ruleList::add) }
        try {
            ruleUtil.createRuleDependencyGraph(ruleList)
        } catch (error: GraphContainsCycleException) {
            log.info("Error: ", error)
            throw ValidatorException(BAD_REQUEST.value(), listOf(createError("rule dependency model contains cycle")))
        }
        return true
    }
    
    companion object {
        private val log = LogManager.getLogger()
    }
}

package io.github.ayushmaanbhav.productFarm.entity.validation

import ValidAttribute
import com.fasterxml.jackson.databind.JsonNode
import io.github.ayushmaanbhav.common.exception.ValidatorException
import io.github.ayushmaanbhav.common.model.response.ErrorDetail
import io.github.ayushmaanbhav.productFarm.constant.AttributeRelationshipType
import io.github.ayushmaanbhav.productFarm.constant.AttributeValueType.FIXED_VALUE
import io.github.ayushmaanbhav.productFarm.constant.AttributeValueType.JUST_DEFINITION
import io.github.ayushmaanbhav.productFarm.constant.AttributeValueType.RULE_DRIVEN
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.constant.DatatypeType.ARRAY
import io.github.ayushmaanbhav.productFarm.constant.DatatypeType.BOOLEAN
import io.github.ayushmaanbhav.productFarm.constant.DatatypeType.INT
import io.github.ayushmaanbhav.productFarm.constant.DatatypeType.NUMBER
import io.github.ayushmaanbhav.productFarm.constant.DatatypeType.OBJECT
import io.github.ayushmaanbhav.productFarm.constant.DatatypeType.STRING
import io.github.ayushmaanbhav.productFarm.entity.Attribute
import io.github.ayushmaanbhav.productFarm.entity.relationship.AbstractAttributeRelatedAttribute
import io.github.ayushmaanbhav.productFarm.entity.repository.AttributeRepo
import io.github.ayushmaanbhav.productFarm.exception.ProductFarmServiceException
import io.github.ayushmaanbhav.productFarm.model.Rule
import io.github.ayushmaanbhav.productFarm.transformer.RuleTransformer
import io.github.ayushmaanbhav.productFarm.util.RuleUtil
import io.github.ayushmaanbhav.productFarm.util.createError
import io.github.ayushmaanbhav.productFarm.util.populateProperty
import jakarta.validation.ConstraintValidator
import jakarta.validation.ConstraintValidatorContext
import org.apache.logging.log4j.LogManager
import org.springframework.http.HttpStatus
import org.springframework.stereotype.Component
import kotlin.reflect.full.declaredMemberProperties

@Component
class AttributeValidator(
    private val ruleValidator: RuleValidator,
    private val abstractAttributeValidator: AbstractAttributeValidator,
    private val attributeGraphValidator: AttributeDirectedAcyclicGraphValidator,
    private val attributeRepo: AttributeRepo,
    private val ruleTransformer: RuleTransformer,
    private val ruleUtil: RuleUtil,
) : ConstraintValidator<ValidAttribute, Attribute> {
    
    override fun isValid(attribute: Attribute, cxt: ConstraintValidatorContext): Boolean {
        val errorList = mutableListOf<ErrorDetail>()
        for (property in Attribute::class.declaredMemberProperties) {
            val errorDetail: ErrorDetail? = when (property) {
                Attribute::abstractAttribute -> isValidAbstractAttribute(attribute, cxt)
                Attribute::displayNames -> isValidDisplayNames(attribute)
                Attribute::path -> createError().takeUnless { attribute.path.let(Constant.PATH_REGEX::matches) }
                Attribute::productId -> null // fk
                Attribute::rule -> idValidRule(attribute, cxt)
                Attribute::type -> isValidType(attribute)
                Attribute::value -> isValidValue(attribute)
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

    private fun isValidValue(attribute: Attribute): ErrorDetail? = createError()
        .takeUnless { attribute.value?.let { attributeValue -> isValidAttributeValue(attribute, attributeValue) } ?: true }

    private fun isValidType(attribute: Attribute): ErrorDetail? = createError()
        .takeUnless {
            when (attribute.type) {
                FIXED_VALUE -> attribute.value != null && attribute.rule == null
                RULE_DRIVEN -> attribute.value == null && attribute.rule != null
                JUST_DEFINITION -> attribute.value == null && attribute.rule == null
            }
        }

    private fun idValidRule(attribute: Attribute, cxt: ConstraintValidatorContext): ErrorDetail? = createError()
        .takeUnless { attribute.rule?.let { rule -> ruleValidator.isValid(rule, cxt) } ?: true }
        .takeUnless {
            attribute.rule?.let { rule ->
                getAllPossibleOutputs(ruleTransformer.forward(rule), attribute.path)
            }?.all { possibleOutput ->
                isValidAttributeValue(attribute, possibleOutput)
            } ?: true
        }
        .takeUnless { attributeGraphValidator.isValid(attribute, cxt) }

    private fun isValidDisplayNames(attribute: Attribute): ErrorDetail? = createError()
        .takeUnless { attribute.displayNames.all { it.id.displayName.let(Constant.DISPLAY_NAME_REGEX::matches) } }

    private fun isValidAbstractAttribute(attribute: Attribute, cxt: ConstraintValidatorContext): ErrorDetail? = createError()
        .takeUnless { abstractAttributeValidator.isValid(attribute.abstractAttribute, cxt) }
        .takeUnless { attribute.abstractAttribute.componentId?.let(Constant.COMPONENT_ID_REGEX::matches) ?: false }

    fun isValidAttributeValue(attribute: Attribute, attributeValue: JsonNode): Boolean {
        val validDatatype = isValidDatatype(attribute, attributeValue)
        val validConstraintRule = isSatisfiesConstraintRuleIfPresent(attribute, attributeValue)
        val validEnumeration = isValidEnumerationIfPresent(attribute, attributeValue)
        val validReference = isValidReferenceIfPresent(attribute, attributeValue)
        return validDatatype && validConstraintRule && validEnumeration && validReference
    }

    private fun isValidReferenceIfPresent(
        attribute: Attribute, attributeValue: JsonNode
    ) = attribute.abstractAttribute.relatedAttributes
        .filter { relationshipsToValidate(it) }
        .flatMap {
            attributeRepo.findAllByAbstractAttribute_AbstractPath(it.id.referenceAbstractPath)
                .map { i -> Pair(it.id.relationship, i) }
        }
        .any { relatedAttributePair ->
            val relationship = relatedAttributePair.first
            val relatedAttribute = relatedAttributePair.second
            val possibleValueNodes = getAllPossibleRelatedAttributeValueNodes(relatedAttribute)
            val possibleValues = getPossibleRelatedAttributeValues(possibleValueNodes, relatedAttribute)
            return when (attribute.abstractAttribute.datatype.type) {
                ARRAY -> isEnumeration(relationship) && attributeValue.all { possibleValues.contains(it.toString()) }
                OBJECT -> attributeValue.fieldNames().asSequence().all {
                    (isKeyEnumeration(relationship) && possibleValues.contains(it))
                        || (isValueEnumeration(relationship) && possibleValues.contains(attributeValue.get(it).toString()))
                }
                else -> isEnumeration(relationship) && possibleValues.contains(attributeValue.toString())
            }
        }

    private fun getPossibleRelatedAttributeValues(
        possibleRelatedAttributeValueNodes: Collection<JsonNode>, relatedAttribute: Attribute
    ): List<String> = possibleRelatedAttributeValueNodes.flatMap {
        when (relatedAttribute.abstractAttribute.datatype.type) {
            ARRAY -> it.map { element -> element.toString() }
            OBJECT -> it.fieldNames().asSequence().toList()
            else -> listOf(it.toString())
        }
    }

    private fun getAllPossibleRelatedAttributeValueNodes(relatedAttribute: Attribute): Collection<JsonNode> =
        when (relatedAttribute.type) {
            FIXED_VALUE -> relatedAttribute.value?.let { listOf(it) } ?: listOf()
            RULE_DRIVEN -> relatedAttribute.rule
                ?.let { rule -> getAllPossibleOutputs(ruleTransformer.forward(rule), relatedAttribute.path) } ?: listOf()
            JUST_DEFINITION -> listOf()
        }

    private fun isValidEnumerationIfPresent(attribute: Attribute, attributeValue: JsonNode): Boolean =
        attribute.abstractAttribute.enumeration?.let { enumeration ->
            when (attribute.abstractAttribute.datatype.type) {
                STRING -> enumeration.values.contains(attributeValue.textValue())
                ARRAY -> attributeValue.all { it.isTextual && enumeration.values.contains(it.textValue()) }
                OBJECT -> attributeValue.fieldNames().asSequence().map { attributeValue.get(it) }
                    .all { it.isTextual && enumeration.values.contains(it.textValue()) }
                else -> false
            }
        } ?: true

    private fun isSatisfiesConstraintRuleIfPresent(attribute: Attribute, attributeValue: JsonNode): Boolean =
        attribute.abstractAttribute.constraintRule?.let { constraintRule ->
            ruleUtil.executeConstraint(ruleTransformer.forward(constraintRule), attributeValue)
        } ?: true

    private fun isValidDatatype(attribute: Attribute, attributeValue: JsonNode): Boolean =
        when (attribute.abstractAttribute.datatype.type) {
            OBJECT -> attributeValue.isObject
            ARRAY -> attributeValue.isArray
            INT -> attributeValue.isIntegralNumber
            NUMBER -> attributeValue.isNumber
            BOOLEAN -> attributeValue.isBoolean
            STRING -> attributeValue.isTextual
        }

    private fun relationshipsToValidate(it: AbstractAttributeRelatedAttribute): Boolean =
        (isEnumeration(it.id.relationship) || isKeyEnumeration(it.id.relationship) || isValueEnumeration(it.id.relationship))

    private fun isValueEnumeration(relationship: String): Boolean = relationship == AttributeRelationshipType.`value-enumeration`.name

    private fun isKeyEnumeration(relationship: String): Boolean = relationship == AttributeRelationshipType.`key-enumeration`.name

    private fun isEnumeration(relationship: String): Boolean = relationship == AttributeRelationshipType.enumeration.name

    fun getAllPossibleOutputs(rule: Rule, path: String): LinkedHashSet<JsonNode> {
        val outputIndex = rule.outputAttributes.indexOf(path)
        val output = LinkedHashSet<List<JsonNode>>()
        rule.displayExpression.returnObject?.let(output::add)
        rule.displayExpression.slab?.defaultReturnObject?.let(output::add)
        rule.displayExpression.slab?.cases?.forEach { it.returnObject?.let(output::add) }
        return output.map { it[outputIndex] }.toCollection(LinkedHashSet())
    }
    
    companion object {
        private val log = LogManager.getLogger()
    }
}

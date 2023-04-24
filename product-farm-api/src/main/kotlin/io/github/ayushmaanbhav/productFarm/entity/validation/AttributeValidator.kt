package io.github.ayushmaanbhav.productFarm.entity.validation

import ValidAttribute
import com.fasterxml.jackson.databind.JsonNode
import io.github.ayushmaanbhav.common.model.response.ErrorDetail
import io.github.ayushmaanbhav.common.exception.ValidatorException
import io.github.ayushmaanbhav.productFarm.constant.AttributeRelationshipType
import io.github.ayushmaanbhav.productFarm.constant.AttributeValueType.DYNAMIC
import io.github.ayushmaanbhav.productFarm.constant.AttributeValueType.STATIC
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.constant.DatatypeType.ARRAY
import io.github.ayushmaanbhav.productFarm.constant.DatatypeType.BOOLEAN
import io.github.ayushmaanbhav.productFarm.constant.DatatypeType.INT
import io.github.ayushmaanbhav.productFarm.constant.DatatypeType.NUMBER
import io.github.ayushmaanbhav.productFarm.constant.DatatypeType.OBJECT
import io.github.ayushmaanbhav.productFarm.constant.DatatypeType.STRING
import io.github.ayushmaanbhav.productFarm.entity.Attribute
import io.github.ayushmaanbhav.productFarm.entity.repository.AttributeRepo
import io.github.ayushmaanbhav.productFarm.exception.ProductFarmServiceException
import io.github.ayushmaanbhav.productFarm.model.Rule
import io.github.ayushmaanbhav.productFarm.transformer.RuleTransformer
import io.github.ayushmaanbhav.productFarm.util.RuleUtil
import io.github.ayushmaanbhav.productFarm.util.createError
import io.github.ayushmaanbhav.productFarm.util.populateProperty
import org.apache.logging.log4j.LogManager
import org.springframework.http.HttpStatus
import org.springframework.stereotype.Component
import jakarta.validation.ConstraintValidator
import jakarta.validation.ConstraintValidatorContext
import kotlin.reflect.full.memberProperties

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
        for (property in Attribute::class.memberProperties) {
            val errorDetail: ErrorDetail? = when (property) {
                Attribute::abstractAttribute ->
                    createError()
                        .takeUnless { abstractAttributeValidator.isValid(attribute.abstractAttribute, cxt) }
                        .takeUnless {
                            attribute.abstractAttribute.componentId?.let(Constant.COMPONENT_ID_REGEX::matches) ?: false
                        }
                Attribute::displayNames ->
                    createError()
                        .takeUnless {
                            attribute.displayNames.all {
                                it.id.displayName.let(Constant.DISPLAY_NAME_REGEX::matches)
                            }
                        }
                Attribute::path ->
                    createError()
                        .takeUnless { attribute.path.let(Constant.PATH_REGEX::matches) }
                Attribute::productId -> null // fk
                Attribute::rule ->
                    createError()
                        .takeUnless {
                            attribute.rule?.let { rule -> ruleValidator.isValid(rule, cxt) } ?: true
                        }
                        .takeUnless {
                            attribute.rule?.let { rule ->
                                getAllPossibleOutputs(ruleTransformer.forward(rule), attribute.path)
                            }?.all { possibleOutput ->
                                isValidAttributeValue(attribute, possibleOutput)
                            } ?: true
                        }
                        .takeUnless { attributeGraphValidator.isValid(attribute, cxt) }
                Attribute::type ->
                    createError()
                        .takeUnless {
                            when (attribute.type) {
                                STATIC -> attribute.value != null && attribute.rule == null
                                DYNAMIC -> attribute.value == null && attribute.rule != null
                            }
                        }
                Attribute::value ->
                    createError()
                        .takeUnless {
                            attribute.value?.let { attributeValue ->
                                isValidAttributeValue(attribute, attributeValue)
                            } ?: true
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
    
    fun isValidAttributeValue(attribute: Attribute, attributeValue: JsonNode): Boolean {
        val validDatatype = when (attribute.abstractAttribute.datatype.type) {
            OBJECT -> attributeValue.isObject
            ARRAY -> attributeValue.isArray
            INT -> attributeValue.isIntegralNumber
            NUMBER -> attributeValue.isNumber
            BOOLEAN -> attributeValue.isBoolean
            STRING -> attributeValue.isTextual
        }
        val validConstraintRule = attribute.abstractAttribute.constraintRule?.let { constraintRule ->
            ruleUtil.executeConstraint(ruleTransformer.forward(constraintRule), attributeValue)
        } ?: true
        val validEnumeration = attribute.abstractAttribute.enumeration?.let { enumeration ->
            when (attribute.abstractAttribute.datatype.type) {
                STRING -> enumeration.values.contains(attributeValue.textValue())
                ARRAY -> attributeValue.all { it.isTextual && enumeration.values.contains(it.textValue()) }
                OBJECT -> attributeValue.fieldNames().asSequence().map { attributeValue.get(it) }
                    .all { it.isTextual && enumeration.values.contains(it.textValue()) }
                else -> false
            }
        } ?: true
        val validReference = attribute.abstractAttribute.relatedAttributes
            .filter {
                it.id.relationship == AttributeRelationshipType.enumeration.name
                || it.id.relationship == AttributeRelationshipType.`key-enumeration`.name
                || it.id.relationship == AttributeRelationshipType.`value-enumeration`.name
            }
            .flatMap {
                attributeRepo.findAllByAbstractAttribute_AbstractPath(it.id.referenceAbstractPath)
                    .map { i -> Pair(it.id.relationship, i) }
            }
            .any { relatedAttributePair ->
                val relationship = relatedAttributePair.first
                val relatedAttribute = relatedAttributePair.second
                val possibleRelatedAttributeValueNodes = when (relatedAttribute.type) {
                    STATIC -> relatedAttribute.value?.let { listOf(it) } ?: listOf()
                    DYNAMIC -> relatedAttribute.rule?.let { rule ->
                        getAllPossibleOutputs(ruleTransformer.forward(rule), relatedAttribute.path)
                    } ?: listOf()
                }
                val possibleRelatedAttributeValues: List<String> = possibleRelatedAttributeValueNodes.flatMap {
                    when (relatedAttribute.abstractAttribute.datatype.type) {
                        ARRAY -> it.map { element -> element.toString() }
                        OBJECT -> it.fieldNames().asSequence().toList()
                        else -> listOf(it.toString())
                    }
                }
                return when (attribute.abstractAttribute.datatype.type) {
                    ARRAY -> relationship == AttributeRelationshipType.enumeration.name && attributeValue
                        .all { possibleRelatedAttributeValues.contains(it.toString()) }
                    OBJECT -> attributeValue.fieldNames().asSequence()
                        .all {
                            (relationship == AttributeRelationshipType.`key-enumeration`.name
                             && possibleRelatedAttributeValues.contains(it))
                            || (relationship == AttributeRelationshipType.`value-enumeration`.name
                                && possibleRelatedAttributeValues.contains(attributeValue.get(it).toString()))
                        }
                    else -> relationship == AttributeRelationshipType.enumeration.name
                            && possibleRelatedAttributeValues.contains(attributeValue.toString())
                }
            }
        return validDatatype && validConstraintRule && validEnumeration && validReference
    }
    
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

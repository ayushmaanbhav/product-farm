package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.api.attribute.dto.CreateAbstractAttributeRequest
import io.github.ayushmaanbhav.productFarm.constant.AttributeRelationshipType
import io.github.ayushmaanbhav.productFarm.entity.AbstractAttribute
import io.github.ayushmaanbhav.productFarm.entity.compositeId.AbstractAttributeRelatedAttributeId
import io.github.ayushmaanbhav.productFarm.entity.compositeId.AbstractAttributeTagId
import io.github.ayushmaanbhav.productFarm.entity.compositeId.AttributeDisplayNameId
import io.github.ayushmaanbhav.productFarm.entity.relationship.AbstractAttributeRelatedAttribute
import io.github.ayushmaanbhav.productFarm.entity.relationship.AbstractAttributeTag
import io.github.ayushmaanbhav.productFarm.entity.relationship.AttributeDisplayName
import io.github.ayushmaanbhav.productFarm.entity.repository.DatatypeRepo
import io.github.ayushmaanbhav.productFarm.entity.repository.ProductRepo
import io.github.ayushmaanbhav.productFarm.entity.repository.ProductTemplateEnumerationRepo
import io.github.ayushmaanbhav.productFarm.util.generateDisplayNames
import io.github.ayushmaanbhav.productFarm.util.generatePath
import io.github.ayushmaanbhav.productFarm.util.getComponentId
import io.github.ayushmaanbhav.productFarm.util.getComponentType
import org.springframework.stereotype.Component

@Component
class CreateAbstractAttributeTransformer(
    private val datatypeRepo: DatatypeRepo,
    private val productRepo: ProductRepo,
    private val enumerationRepo: ProductTemplateEnumerationRepo,
    private val createRuleTransformer: CreateRuleTransformer,
) : OneWayTransformer<Pair<String, CreateAbstractAttributeRequest>, AbstractAttribute> {
    
    override fun forward(input: Pair<String, CreateAbstractAttributeRequest>): AbstractAttribute {
        val productId = input.first
        val request = input.second
        val componentType = getComponentType(request.componentType)
        val componentId = request.componentId?.let { getComponentId(it) }
        val abstractPath = generatePath(productId, request.componentType, request.componentId, request.name)
        val displayNames = generateDisplayNames(productId, request.componentType, request.componentId, request.name)
        val datatype = datatypeRepo.getReferenceById(request.datatype)
        val product = productRepo.getReferenceById(productId)
        val enumeration = request.enumeration?.let {
            enumerationRepo.getByProductTemplateTypeAndName(product.templateType, it)
        }
        return AbstractAttribute(
            abstractPath = abstractPath,
            displayNames = displayNames.mapIndexed { index, displayNamePair ->
                AttributeDisplayName(
                    id = AttributeDisplayNameId(
                        productId = productId,
                        displayName = displayNamePair.second,
                    ),
                    abstractPath = abstractPath,
                    path = null,
                    displayNameFormat = displayNamePair.first,
                    order = index
                )
            },
            componentType = componentType,
            componentId = componentId,
            tags = request.tags.mapIndexed { index, tag ->
                AbstractAttributeTag(
                    id = AbstractAttributeTagId(
                        abstractPath = abstractPath,
                        tag = tag
                    ),
                    productId = productId,
                    order = index
                )
            },
            datatype = datatype,
            enumeration = enumeration,
            relatedAttributes = request.relatedAttributes?.mapIndexed { index, relatedAttribute ->
                AbstractAttributeRelatedAttribute(
                    id = AbstractAttributeRelatedAttributeId(
                        abstractPath = abstractPath,
                        referenceAbstractPath = relatedAttribute,
                        relationship = AttributeRelationshipType.enumeration.name,
                    ),
                    order = index
                )
            } ?: listOf(),
            constraintRule = request.constraintExpression?.let { createRuleTransformer.forward(it) },
            immutable = request.immutable,
            description = request.description,
            productId = productId,
        )
    }
}

package io.github.ayushmaanbhav.productFarm.service

import io.github.ayushmaanbhav.productFarm.api.attribute.dto.CreateAttributeRequest
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAttributeListByTagResponse
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAttributeResponse
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetFunctionalityAttributeListResponse
import io.github.ayushmaanbhav.productFarm.entity.compositeId.AttributeDisplayNameId
import io.github.ayushmaanbhav.productFarm.entity.repository.AbstractAttributeTagRepo
import io.github.ayushmaanbhav.productFarm.entity.repository.AttributeDisplayNameRepo
import io.github.ayushmaanbhav.productFarm.entity.repository.AttributeRepo
import io.github.ayushmaanbhav.productFarm.transformer.GetAttributeTransformer
import org.springframework.stereotype.Component
import java.util.*

@Component
class AttributeService(
    private val getAttributeTransformer: GetAttributeTransformer,
    private val
    private val attributeRepo: AttributeRepo,
    private val attributeDisplayNameRepo: AttributeDisplayNameRepo,
    private val abstractAttributeTagRepo: AbstractAttributeTagRepo,
) {
    fun create(productId: String, request: CreateAttributeRequest) {

    }
    
    fun get(productId: String, displayName: String): Optional<GetAttributeResponse> {
        return attributeDisplayNameRepo.findById(AttributeDisplayNameId(productId, displayName))
            .flatMap { it.path?.let { it1 -> attributeRepo.findById(it1) } }
            .map { getAttributeTransformer.forward(it) }
    }
    
    fun getFunctionalityAttribute(
        productId: String, functionality: String
    ): Optional<GetFunctionalityAttributeListResponse> {

    }
    
    fun getAttributeByTag(productId: String, tag: String): Optional<GetAttributeListByTagResponse> {
        return abstractAttributeTagRepo.getByProductIdAndTag(productId, tag)
            .flatMap { it.id.abstractPath.let { it1 -> attributeRepo.findAllByAbstractAttribute_AbstractPath(it1) } }
            .map { getAttributeTransformer.forward(it) }
    }
    
    fun clone(parentProductId: String, productId: String) {
        TODO()
    }
}

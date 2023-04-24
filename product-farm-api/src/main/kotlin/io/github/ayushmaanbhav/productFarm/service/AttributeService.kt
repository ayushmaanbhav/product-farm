package io.github.ayushmaanbhav.productFarm.service

import io.github.ayushmaanbhav.productFarm.api.attribute.dto.CreateAttributeRequest
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAttributeListByTagResponse
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAttributeResponse
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetFunctionalityAttributeListResponse
import io.github.ayushmaanbhav.productFarm.entity.compositeId.AttributeDisplayNameId
import io.github.ayushmaanbhav.productFarm.entity.repository.AbstractAttributeRepo
import io.github.ayushmaanbhav.productFarm.entity.repository.AttributeDisplayNameRepo
import io.github.ayushmaanbhav.productFarm.entity.repository.AttributeRepo
import io.github.ayushmaanbhav.productFarm.entity.repository.ProductRepo
import io.github.ayushmaanbhav.productFarm.transformer.GetAttributeTransformer
import org.springframework.stereotype.Component
import java.util.*

@Component
class AttributeService(
    private val getAttributeTransformer: GetAttributeTransformer,
    private val attributeRepo: AttributeRepo,
    private val productRepo: ProductRepo,
    private val abstractAttributeRepo: AbstractAttributeRepo,
    private val attributeDisplayNameRepo: AttributeDisplayNameRepo,
) {
    fun create(productId: String, request: CreateAttributeRequest) {
    
    }
    
    fun get(productId: String, displayName: String): Optional<GetAttributeResponse> {
        return attributeDisplayNameRepo.findById(AttributeDisplayNameId(productId, displayName))
            .flatMap { it.path?.let { it1 -> attributeRepo.findById(it1) } }
            .map { attribute.forward(it) }
    }
    
    fun getFunctionalityAttribute(
        productId: String, functionality: String
    ): Optional<GetFunctionalityAttributeListResponse> {
    }
    
    fun getAttributeByTag(productId: String, tag: String): Optional<GetAttributeListByTagResponse> {
    }
    
    fun clone(parentProductId: String, productId: String) {
        TODO()
    }
}

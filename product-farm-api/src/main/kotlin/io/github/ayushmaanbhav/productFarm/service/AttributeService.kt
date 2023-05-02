package io.github.ayushmaanbhav.productFarm.service

import io.github.ayushmaanbhav.common.exception.ValidatorException
import io.github.ayushmaanbhav.common.model.response.ErrorDetail
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.CreateAttributeRequest
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAttributeListByTagResponse
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAttributeResponse
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetFunctionalityAttributeListResponse
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.entity.compositeId.AttributeDisplayNameId
import io.github.ayushmaanbhav.productFarm.entity.repository.AbstractAttributeRepo
import io.github.ayushmaanbhav.productFarm.entity.repository.AbstractAttributeTagRepo
import io.github.ayushmaanbhav.productFarm.entity.repository.AttributeDisplayNameRepo
import io.github.ayushmaanbhav.productFarm.entity.repository.AttributeRepo
import io.github.ayushmaanbhav.productFarm.entity.repository.ProductRepo
import io.github.ayushmaanbhav.productFarm.transformer.CreateAttributeTransformer
import io.github.ayushmaanbhav.productFarm.transformer.GetAttributeByTagTransformer
import io.github.ayushmaanbhav.productFarm.transformer.GetAttributeTransformer
import io.github.ayushmaanbhav.productFarm.util.createError
import io.github.ayushmaanbhav.productFarm.util.dissectAttributeDisplayName
import io.github.ayushmaanbhav.productFarm.util.generatePath
import org.springframework.stereotype.Component
import java.util.*
import org.springframework.http.HttpStatus
import kotlin.collections.LinkedHashSet

@Component
class AttributeService(
    private val createAttributeTransformer: CreateAttributeTransformer,
    private val getAttributeTransformer: GetAttributeTransformer,
    private val getAttributeByTagTransformer: GetAttributeByTagTransformer,
    private val productRepo: ProductRepo,
    private val abstractAttributeRepo: AbstractAttributeRepo,
    private val attributeRepo: AttributeRepo,
    private val attributeDisplayNameRepo: AttributeDisplayNameRepo,
    private val abstractAttributeTagRepo: AbstractAttributeTagRepo,
) {
    fun create(productId: String, request: CreateAttributeRequest) {
        validateCreateRequest(productId, request)
        attributeRepo.save(createAttributeTransformer.forward(Pair(productId, request)))
    }
    
    fun get(productId: String, displayName: String): Optional<GetAttributeResponse> {
        return attributeDisplayNameRepo.findById(AttributeDisplayNameId(productId, displayName))
            .flatMap { it.path?.let { it1 -> attributeRepo.findById(it1) } }
            .map { getAttributeTransformer.forward(it) }
    }
    
    fun getFunctionalityAttribute(productId: String, functionality: String): Optional<GetFunctionalityAttributeListResponse> {
        TODO()
    }
    
    fun getAttributeByTag(productId: String, tag: String): Optional<GetAttributeListByTagResponse> {
        return abstractAttributeTagRepo.getByProductIdAndTag(productId, tag)
            .flatMap { it.id.abstractPath.let { it1 -> attributeRepo.findAllByAbstractAttribute_AbstractPath(it1) } }
            .map { getAttributeByTagTransformer.forward(it) }
            .let { Optional.of(GetAttributeListByTagResponse(it.toCollection(LinkedHashSet()))) }
    }
    
    fun clone(parentProductId: String, productId: String) {
        TODO()
    }

    private fun validateCreateRequest(productId: String, request: CreateAttributeRequest) {
        val dissectedAttributeId = dissectAttributeDisplayName(request.displayName)
            ?: throw ValidatorException(HttpStatus.BAD_REQUEST.value(), "Invalid display name provided")
        listOf(
            generatePath(productId, dissectedAttributeId.componentType, dissectedAttributeId.componentId, dissectedAttributeId.name),
            generatePath(productId, dissectedAttributeId.componentType, null, dissectedAttributeId.name)
        ).firstOrNull(abstractAttributeRepo::existsById)
            ?: throw ValidatorException(HttpStatus.BAD_REQUEST.value(), "Abstract attribute does not exist")
        val errorList = mutableListOf<ErrorDetail>()
        if (productRepo.existsById(productId).not()) {
            errorList.add(createError("Product does not exist for this id"))
        }
        if (Constant.ORIGINAL_COMPONENT_TYPE_REGEX.matches(dissectedAttributeId.componentType).not()) {
            errorList.add(createError("Please enter a valid componentType"))
        }
        if (Constant.ORIGINAL_COMPONENT_ID_REGEX.matches(dissectedAttributeId.componentId).not()) {
            errorList.add(createError("Please enter a valid componentId"))
        }
        if (Constant.ORIGINAL_ATTRIBUTE_NAME_REGEX.matches(dissectedAttributeId.name).not()) {
            errorList.add(createError("Please enter a valid name"))
        }
        if ((request.value == null && request.rule == null) || (request.value != null && request.rule != null)) {
            errorList.add(createError("Please specify either a value or a rule, cant be both empty or both specified"))
        }
        if (errorList.isNotEmpty()) {
            throw ValidatorException(HttpStatus.BAD_REQUEST.value(), errorList)
        }
    }
}

package io.github.ayushmaanbhav.productFarm.service

import io.github.ayushmaanbhav.common.exception.ValidatorException
import io.github.ayushmaanbhav.common.model.response.ErrorDetail
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.CreateAbstractAttributeRequest
import io.github.ayushmaanbhav.productFarm.api.attribute.dto.GetAbstractAttributeResponse
import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.entity.compositeId.AttributeDisplayNameId
import io.github.ayushmaanbhav.productFarm.entity.repository.AbstractAttributeRepo
import io.github.ayushmaanbhav.productFarm.entity.repository.AttributeDisplayNameRepo
import io.github.ayushmaanbhav.productFarm.entity.repository.DatatypeRepo
import io.github.ayushmaanbhav.productFarm.entity.repository.ProductRepo
import io.github.ayushmaanbhav.productFarm.entity.repository.ProductTemplateEnumerationRepo
import io.github.ayushmaanbhav.productFarm.transformer.CreateAbstractAttributeTransformer
import io.github.ayushmaanbhav.productFarm.transformer.GetAbstractAttributeTransformer
import io.github.ayushmaanbhav.productFarm.util.createError
import io.github.ayushmaanbhav.productFarm.util.generatePath
import jakarta.transaction.Transactional
import java.util.*
import org.springframework.http.HttpStatus
import org.springframework.stereotype.Component

@Component
class AbstractAttributeService(
    private val createAbstractAttributeTransformer: CreateAbstractAttributeTransformer,
    private val getAbstractAttributeTransformer: GetAbstractAttributeTransformer,
    private val abstractAttributeRepo: AbstractAttributeRepo,
    private val datatypeRepo: DatatypeRepo,
    private val productRepo: ProductRepo,
    private val enumerationRepo: ProductTemplateEnumerationRepo,
    private val attributeDisplayNameRepo: AttributeDisplayNameRepo,
) {
    @Transactional
    fun create(productId: String, request: CreateAbstractAttributeRequest) {
        validateCreateRequest(productId, request)
        abstractAttributeRepo.save(createAbstractAttributeTransformer.forward(Pair(productId, request)))
    }
    
    fun get(productId: String, displayName: String): Optional<GetAbstractAttributeResponse> {
        return attributeDisplayNameRepo.findById(AttributeDisplayNameId(productId, displayName))
            .flatMap { it.abstractPath?.let { it1 -> abstractAttributeRepo.findById(it1) } }
            .map { getAbstractAttributeTransformer.forward(it) }
    }
    
    fun clone(parentProductId: String, productId: String) {
        TODO()
    }
    
    private fun validateCreateRequest(productId: String, request: CreateAbstractAttributeRequest) {
        val errorList = mutableListOf<ErrorDetail>()
        val abstractPath = generatePath(productId, request.componentType, request.componentId, request.name)
        if (abstractAttributeRepo.existsById(abstractPath)) {
            errorList.add(createError("Abstract attribute already exists for this id"))
        }
        if (productRepo.existsById(productId).not()) {
            errorList.add(createError("Product does not exist for this id"))
        }
        val product = productRepo.getReferenceById(productId)
        if (datatypeRepo.existsById(request.datatype).not()) {
            errorList.add(createError("Datatype does not exist for this id"))
        }
        if (request.enumeration != null
            && enumerationRepo.existsByProductTemplateTypeAndName(product.templateType, request.enumeration).not()
        ) {
            errorList.add(createError("Enumeration does not exist for this id"))
        }
        request.relatedAttributes?.filter {
            abstractAttributeRepo.existsById(it).not()
        }?.map {
            errorList.add(createError("Related attribute does not exist for this id: $it"))
        }
        if (Constant.ORIGINAL_COMPONENT_TYPE_REGEX.matches(request.componentType).not()) {
            errorList.add(createError("Please enter a valid componentType"))
        }
        if (request.componentId?.let { Constant.ORIGINAL_COMPONENT_ID_REGEX.matches(it).not() } == true) {
            errorList.add(createError("Please enter a valid componentId"))
        }
        if (Constant.ORIGINAL_ATTRIBUTE_NAME_REGEX.matches(request.name).not()) {
            errorList.add(createError("Please enter a valid name"))
        }
        if (errorList.isNotEmpty()) {
            throw ValidatorException(HttpStatus.BAD_REQUEST.value(), errorList)
        }
    }
}

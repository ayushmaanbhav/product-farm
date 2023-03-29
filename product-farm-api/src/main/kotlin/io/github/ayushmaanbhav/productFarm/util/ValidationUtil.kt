package io.github.ayushmaanbhav.productFarm.validation

import io.github.ayushmaanbhav.common.model.response.ErrorDetail
import kotlin.reflect.KProperty1

fun createError(): ErrorDetail = ErrorDetail.builder().build()

fun createError(message: String): ErrorDetail = ErrorDetail.builder().message(message).build()

fun populateProperty(errorDetail: ErrorDetail, property: KProperty1<*, *>): ErrorDetail {
    val errorDetailBuilder = ErrorDetail.builder()
        .params(errorDetail.params)
        .code(errorDetail.code)
        .param("property", property.name)
    errorDetail.message?.let { errorDetailBuilder.message(it) }
    return errorDetailBuilder.build()
}

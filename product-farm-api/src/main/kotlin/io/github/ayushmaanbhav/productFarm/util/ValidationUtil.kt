package io.github.ayushmaanbhav.productFarm.util

import io.github.ayushmaanbhav.common.model.response.ErrorDetail
import kotlin.reflect.KProperty1

fun createError(): ErrorDetail = ErrorDetail()

fun createError(message: String): ErrorDetail = ErrorDetail(message = message)

fun populateProperty(errorDetail: ErrorDetail, property: KProperty1<*, *>): ErrorDetail {
    val params = errorDetail.params?.let { HashMap(it) } ?: HashMap()
    params["property"] = property.name
    return ErrorDetail(
        code = errorDetail.code,
        params = params,
        message = errorDetail.message
    )
}

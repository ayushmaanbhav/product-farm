package io.github.ayushmaanbhav.common.model.response

data class ErrorDetail(val code: String? = null, val message: String? = null, val params: Map<String, String>? = null)

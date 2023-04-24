package io.github.ayushmaanbhav.common.exception

import io.github.ayushmaanbhav.common.model.response.ErrorDetail

open class ValidatorException(val code: Int, message: String?, throwable: Throwable?, val errors: List<ErrorDetail>?) : Exception(message, throwable) {
    constructor(code: Int, message: String) : this(code, message, null, null)
    constructor(code: Int, errors: List<ErrorDetail>) : this(code, null, null, errors)
    constructor(code: Int, message: String, errors: List<ErrorDetail>) : this(code, message, null, errors)
    constructor(code: Int, message: String, throwable: Throwable) : this(code, message, throwable, null)
}

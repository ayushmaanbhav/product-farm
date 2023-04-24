package io.github.ayushmaanbhav.common.exception

open class ServiceException(message: String, throwable: Throwable?, val errors: List<String>?) : Exception(message, throwable) {
    constructor(message: String) : this(message, null, null)
    constructor(message: String, errors: List<String>) : this(message, null, errors)
    constructor(message: String, throwable: Throwable) : this(message, throwable, null)
}

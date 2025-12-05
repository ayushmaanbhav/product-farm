package io.github.ayushmaanbhav.productFarm.exception

import io.github.ayushmaanbhav.common.exception.ServiceException

class ProductFarmServiceException : ServiceException {
    constructor(message: String) : super(message)
    constructor(message: String, errors: Array<String>) : super(message, errors.asList())
    constructor(message: String, exception: Exception) : super(message, exception)
}

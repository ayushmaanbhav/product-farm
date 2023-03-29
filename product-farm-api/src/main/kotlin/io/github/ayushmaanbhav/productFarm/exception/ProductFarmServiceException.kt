package io.github.ayushmaanbhav.productFarm.exception

import io.github.ayushmaanbhav.common.exception.base.ServiceException

class ProductFarmServiceException : ServiceException {
    constructor(message: String) : super(message)
    
    constructor(message: String, errors: Array<String>) : super(message, *errors)
    
    constructor(message: String, exception: Exception) : super(message, exception)
}

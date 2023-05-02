package io.github.ayushmaanbhav.productFarm.transformer

import io.github.ayushmaanbhav.productFarm.exception.ProductFarmServiceException

interface OneWayTransformer<I, O> : Transformer<I, O> {
    override fun reverse(input: O): I = throw ProductFarmServiceException("Operation not supported")
}

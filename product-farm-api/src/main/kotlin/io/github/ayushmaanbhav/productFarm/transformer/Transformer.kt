package io.github.ayushmaanbhav.productFarm.transformer

import kotlin.reflect.KClass
import kotlin.reflect.full.createInstance

sealed interface Transformer<I, O> {
    fun forward(input: I): O
    
    fun reverse(input: O): I
    
    fun <OT : MutableCollection<O>> forward(input: Collection<I>, outputType: KClass<OT>): OT =
        input.map(this::forward).toCollection(outputType.createInstance())
    
    fun <IT : MutableCollection<I>> reverse(input: Collection<O>, outputType: KClass<IT>): IT =
        input.map(this::reverse).toCollection(outputType.createInstance())
}

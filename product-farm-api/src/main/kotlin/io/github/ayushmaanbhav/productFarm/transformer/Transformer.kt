package io.github.ayushmaanbhav.productFarm.transformer

import kotlin.reflect.KClass
import kotlin.reflect.full.createInstance

abstract class Transformer<I, O> {
    abstract fun forward(input: I): O
    
    abstract fun reverse(input: O): I
    
    fun <OT : MutableCollection<O>> forward(input: Collection<I>, outputType: KClass<OT>): OT =
        input.map(this::forward).toCollection(outputType.createInstance())
    
    fun <IT : MutableCollection<I>> reverse(input: Collection<O>, outputType: KClass<IT>): IT =
        input.map(this::reverse).toCollection(outputType.createInstance())
}

package io.github.ayushmaanbhav.jsonLogic.utils

import java.math.BigDecimal
import io.github.ayushmaanbhav.jsonLogic.config.MathContext
import io.github.ayushmaanbhav.jsonLogic.utils.type.JsonLogicList

val Any?.asList: List<Any?>
    get() = (this as? List<*>)?.let {
        JsonLogicList(it)
    } ?: JsonLogicList(listOf(this))

val List<Any?>.comparableList: List<Comparable<*>?>
    get() = asList.map { it.asComparable }

private val Any?.asComparable: Comparable<*>?
    get() = when (this) {
        is Comparable<*> -> this
        is List<*> -> JsonLogicList(this)
        else -> null
    }

val Any?.asBigDecimalList: List<BigDecimal?>
    get() = asList.map { it.toBigDecimalOrNull() }

fun Any?.asBigDecimalList(mc: MathContext): List<BigDecimal?> =
    asList.map { it.toBigDecimalOrNull(mc) }

fun Any?.toStringOrEmpty() = this?.let { toString() }.orEmpty()

fun Any?.toStringOrNull() = this?.let { toString() }

fun Any?.toBigDecimalOrNull(): BigDecimal? = this?.let { if (isNumeric()) toBigDecimal() else null }

fun Any?.toBigDecimalOrNull(mc: MathContext) = this?.let { if (isNumeric()) toBigDecimal(mc) else null }

fun Any.toBigDecimal() = this as? BigDecimal ?: BigDecimal(toString())

fun Any.toBigDecimal(mc: MathContext): BigDecimal = (this as? BigDecimal ?: BigDecimal(toString(), mc.mc())).setScale(mc)

fun BigDecimal.setScale(mc: MathContext): BigDecimal = setScale(mc.scale, mc.roundingMode)

fun Any?.isSingleNullList() = this is List<*> && size == 1 && first() == null

fun Any?.isExpression() = (this as? Map<*, *>)?.let {
    it.isNotEmpty() && it.keys.all { key -> key is String }
} ?: false

fun Number.isWhole() = toBigDecimal().stripTrailingZeros().scale() <= 0

fun Any.toBigDecimalDefaultContext(): BigDecimal =
    BigDecimal(toString(), getDefaultMathContext().mc()).setScale(getDefaultMathContext())

fun List<Any>.toBigDecimalDefaultContextList(): List<BigDecimal> = map {
    BigDecimal(it.toString(), getDefaultMathContext().mc()).setScale(getDefaultMathContext())
}

fun getDefaultMathContext() = MathContext.DEFAULT

private fun Any.isNumeric() = this is Number || toString().toDoubleOrNull() != null

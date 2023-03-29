package io.github.ayushmaanbhav.jsonLogic.utils

import java.math.BigDecimal

val String.intOrZero: Int
    get() = bigDecimalOrZero.toInt()

val String.longOrZero: Long
    get() = bigDecimalOrZero.toLong()

val String.bigDecimalOrZero: BigDecimal
    get() = toBigDecimalOrNull() ?: BigDecimal.ZERO


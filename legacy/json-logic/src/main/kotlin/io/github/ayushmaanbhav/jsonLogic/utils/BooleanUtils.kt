package io.github.ayushmaanbhav.jsonLogic.utils

import java.math.BigDecimal

fun Boolean.asNumber() = if (this) 1.0 else 0.0

fun Boolean.asBigDecimal(): BigDecimal = if (this) BigDecimal.ONE else BigDecimal.ZERO

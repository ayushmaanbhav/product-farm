package io.github.ayushmaanbhav.jsonLogic.config

import java.math.RoundingMode

data class MathContext(val scale: Int, val precision: Int, val roundingMode: RoundingMode) {
    constructor(scale: Int, roundingMode: RoundingMode) : this(scale, DEFAULT.precision, roundingMode)

    fun mc() = java.math.MathContext(precision, roundingMode)

    companion object {
        val DEFAULT = MathContext(64, 0, RoundingMode.HALF_UP)
    }
}

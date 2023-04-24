package io.github.ayushmaanbhav.common.model

import java.math.BigDecimal
import java.math.BigInteger
import java.math.RoundingMode
import org.joda.money.BigMoney
import org.joda.money.CurrencyUnit

data class Money(val value: String, val scale: Int, val roundingMode: RoundingMode, val currency: String) : Comparable<Money> {
    init {
        require(value.isWholeNumber()) { "value should be a whole number" }
        require(scale >= 0) { "scale should be >= 0" }
    }

    fun toBigMoney(): BigMoney = BigMoney.of(CurrencyUnit.of(currency), BigDecimal(value).movePointLeft(scale))

    override operator fun compareTo(other: Money): Int = toBigMoney().compareTo(other.toBigMoney())

    operator fun plus(o: Money): Money = of(toBigMoney().plus(o.toBigMoney()), scale, roundingMode)

    operator fun minus(o: Money): Money = of(toBigMoney().minus(o.toBigMoney()), scale, roundingMode)

    operator fun times(valueToMultiplyBy: BigDecimal): Money =
        of(toBigMoney().multiplyRetainScale(valueToMultiplyBy, roundingMode), scale, roundingMode)

    operator fun div(valueToDivideBy: BigDecimal): Money =
        of(toBigMoney().dividedBy(valueToDivideBy, roundingMode), scale, roundingMode)

    private fun String.isWholeNumber(): Boolean = value.runCatching { BigInteger(value) }.isSuccess

    companion object {
        fun of(bigMoney: BigMoney, scale: Int, roundingMode: RoundingMode): Money = Money(
            bigMoney.amount.setScale(scale, roundingMode).movePointRight(scale).toPlainString(),
            scale, roundingMode, bigMoney.currencyUnit.code
        )
    }
}

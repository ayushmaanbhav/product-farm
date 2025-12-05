package io.github.ayushmaanbhav.productFarm.util

import org.apache.logging.log4j.util.Strings

enum class CaseType(
    val separator: String,
    val upperCase: Boolean,
    val firstWordLetterOpposite: Boolean,
    val firstLetterOpposite: Boolean
) {
    `lower-dash-case`(separator = "-", upperCase = false, firstWordLetterOpposite = false, firstLetterOpposite = false),
    `UPPER-DASH-CASE`(separator = "-", upperCase = true, firstWordLetterOpposite = false, firstLetterOpposite = false),
    UPPER_SNAKE_CASE(separator = "_", upperCase = true, firstWordLetterOpposite = false, firstLetterOpposite = false),
    lower_snake_case(separator = "_", upperCase = false, firstWordLetterOpposite = false, firstLetterOpposite = false),
    lowerCamelCase(separator = "", upperCase = false, firstWordLetterOpposite = true, firstLetterOpposite = false),
    UpperCamelCase(separator = "", upperCase = false, firstWordLetterOpposite = true, firstLetterOpposite = true)
}

private val IGNORE_REGEX = "[^a-zA-Z0-9_\\s\\-]+".toRegex()
private val REPLACE_WITH_SEPARATOR_REGEX = "[_\\s-]+".toRegex()

private fun toggleFirstLetterCase(word: String): String {
    val firstOpposite = if (word[0].isUpperCase()) word[0].lowercase() else word[0].uppercase()
    return if (word.length > 1) firstOpposite.plus(word.substring(1)) else firstOpposite
}

private fun tryDetectCamelCase(value: String, separator: String): String {
    var i = 0
    var resultVal = value[i].toString()
    while (++i < value.length) {
        if ((value[i - 1].isLowerCase() && value[i].isUpperCase()
             && (i == value.length - 1 || value[i + 1].isLowerCase()))
            || (value[i - 1].isUpperCase() && value[i].isLowerCase()
                && (i == value.length - 1 || value[i + 1].isUpperCase()))
        ) {
            resultVal += separator + value[i] + if (i == value.length - 1) "" else value[i + 1]
            ++i
        } else {
            resultVal += value[i]
        }
    }
    return resultVal
}

fun String.convertCaseConvention(caseType: CaseType): String {
    val separator = "-"
    var value = trim().replace(IGNORE_REGEX, Strings.EMPTY)
        .replace(REPLACE_WITH_SEPARATOR_REGEX, separator)
    if (value.length == length) {
        value = tryDetectCamelCase(value, separator)
    }
    value = if (caseType.upperCase) value.uppercase() else value.lowercase()
    if (caseType.firstWordLetterOpposite) {
        value = value.split(separator).mapIndexed { index, word ->
            if (index == 0 && caseType.firstLetterOpposite.not()) return@mapIndexed word
            return@mapIndexed toggleFirstLetterCase(word)
        }.joinToString(Strings.EMPTY)
    }
    return value.replace(separator, caseType.separator)
}

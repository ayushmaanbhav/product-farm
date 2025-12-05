package io.github.ayushmaanbhav.jsonLogic.stdlib

import io.github.ayushmaanbhav.jsonLogic.api.operation.FunctionalLogicOperation
import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.stdlib.array.Distinct
import io.github.ayushmaanbhav.jsonLogic.stdlib.array.Find
import io.github.ayushmaanbhav.jsonLogic.stdlib.array.JoinToString
import io.github.ayushmaanbhav.jsonLogic.stdlib.array.Size
import io.github.ayushmaanbhav.jsonLogic.stdlib.array.Sort
import io.github.ayushmaanbhav.jsonLogic.stdlib.encoding.Encode
import io.github.ayushmaanbhav.jsonLogic.stdlib.format.DecimalFormat
import io.github.ayushmaanbhav.jsonLogic.stdlib.string.Capitalize
import io.github.ayushmaanbhav.jsonLogic.stdlib.string.IsBlank
import io.github.ayushmaanbhav.jsonLogic.stdlib.string.Length
import io.github.ayushmaanbhav.jsonLogic.stdlib.string.Lowercase
import io.github.ayushmaanbhav.jsonLogic.stdlib.string.Match
import io.github.ayushmaanbhav.jsonLogic.stdlib.string.Replace
import io.github.ayushmaanbhav.jsonLogic.stdlib.string.ToArray
import io.github.ayushmaanbhav.jsonLogic.stdlib.string.Trim
import io.github.ayushmaanbhav.jsonLogic.stdlib.string.Uppercase

object OperationsProvider {
    val standardOperations: Map<String, StandardLogicOperation> = mutableMapOf(
        // string
        "capitalize" to Capitalize,
        "isBlank" to IsBlank,
        "length" to Length,
        "lowercase" to Lowercase,
        "replace" to Replace,
        "uppercase" to Uppercase,
        "toArray" to ToArray,
        "decimalFormat" to DecimalFormat,
        "encode" to Encode,
        "match" to Match,

        // time
        "currentTime" to CurrentTimeMillis,

        // array
        "size" to Size,
        "sort" to Sort,
        "distinct" to Distinct,
        "joinToString" to JoinToString,

        "drop" to Drop,
        "reverse" to Reverse,
        "trim" to Trim
    )

    val functionalOperations: Map<String, FunctionalLogicOperation> = mutableMapOf(
        "find" to Find,
    )
}

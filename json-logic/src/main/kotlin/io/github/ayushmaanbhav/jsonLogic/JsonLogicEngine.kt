package io.github.ayushmaanbhav.jsonLogic

import io.github.ayushmaanbhav.jsonLogic.api.operation.FunctionalLogicOperation
import io.github.ayushmaanbhav.jsonLogic.api.operation.StandardLogicOperation
import io.github.ayushmaanbhav.jsonLogic.config.StandardLogicOperationConfig
import io.github.ayushmaanbhav.jsonLogic.config.StreamProcessingConfig
import io.github.ayushmaanbhav.jsonLogic.evaluation.CommonLogicEvaluator
import io.github.ayushmaanbhav.jsonLogic.evaluation.LogicOperations
import io.github.ayushmaanbhav.jsonLogic.operations.In
import io.github.ayushmaanbhav.jsonLogic.operations.Log
import io.github.ayushmaanbhav.jsonLogic.operations.array.Filter
import io.github.ayushmaanbhav.jsonLogic.operations.array.Merge
import io.github.ayushmaanbhav.jsonLogic.operations.array.Reduce
import io.github.ayushmaanbhav.jsonLogic.operations.array.occurence.All
import io.github.ayushmaanbhav.jsonLogic.operations.array.occurence.None
import io.github.ayushmaanbhav.jsonLogic.operations.array.occurence.Some
import io.github.ayushmaanbhav.jsonLogic.operations.data.Missing
import io.github.ayushmaanbhav.jsonLogic.operations.data.MissingSome
import io.github.ayushmaanbhav.jsonLogic.operations.data.Var
import io.github.ayushmaanbhav.jsonLogic.operations.logic.And
import io.github.ayushmaanbhav.jsonLogic.operations.logic.DoubleNegation
import io.github.ayushmaanbhav.jsonLogic.operations.logic.If
import io.github.ayushmaanbhav.jsonLogic.operations.logic.Negation
import io.github.ayushmaanbhav.jsonLogic.operations.logic.Or
import io.github.ayushmaanbhav.jsonLogic.operations.logic.equals.Equals
import io.github.ayushmaanbhav.jsonLogic.operations.logic.equals.NotEquals
import io.github.ayushmaanbhav.jsonLogic.operations.logic.equals.strict.NotStrictEquals
import io.github.ayushmaanbhav.jsonLogic.operations.logic.equals.strict.StrictEquals
import io.github.ayushmaanbhav.jsonLogic.operations.numeric.Addition
import io.github.ayushmaanbhav.jsonLogic.operations.numeric.Division
import io.github.ayushmaanbhav.jsonLogic.operations.numeric.Max
import io.github.ayushmaanbhav.jsonLogic.operations.numeric.Min
import io.github.ayushmaanbhav.jsonLogic.operations.numeric.Modulo
import io.github.ayushmaanbhav.jsonLogic.operations.numeric.Multiplication
import io.github.ayushmaanbhav.jsonLogic.operations.numeric.Subtraction
import io.github.ayushmaanbhav.jsonLogic.operations.numeric.compare.GreaterThan
import io.github.ayushmaanbhav.jsonLogic.operations.numeric.compare.GreaterThanOrEqualTo
import io.github.ayushmaanbhav.jsonLogic.operations.numeric.compare.LessThan
import io.github.ayushmaanbhav.jsonLogic.operations.numeric.compare.LessThanOrEqualTo
import io.github.ayushmaanbhav.jsonLogic.operations.string.Cat
import io.github.ayushmaanbhav.jsonLogic.operations.string.Substr
import io.github.ayushmaanbhav.jsonLogic.stream.JsonLogicStreamProcessor
import java.io.InputStream
import kotlin.collections.Map
import io.github.ayushmaanbhav.jsonLogic.operations.array.Map as LogicMap

interface JsonLogicEngine {
    fun evaluate(expression: Map<String, Any?>, data: Any?): JsonLogicResult

    fun evaluate(inputStream: InputStream, data: Any?): JsonLogicResult {
        throw UnsupportedOperationException("Not supported by default, pls set the streaming flag")
    }

    class Builder {
        private var logger: ((Any?) -> Unit)? = null
        private var standardLogicOperationConfig: StandardLogicOperationConfig = StandardLogicOperationConfig.DEFAULT
        private var streamProcessingConfig: StreamProcessingConfig = StreamProcessingConfig.DEFAULT
        private val standardOperations: MutableMap<String, StandardLogicOperation> = mutableMapOf(
            // data
            "var" to Var,
            "missing_some" to MissingSome,
            "missing" to Missing,

            // numeric
            ">" to GreaterThan,
            ">=" to GreaterThanOrEqualTo,
            "<" to LessThan,
            "<=" to LessThanOrEqualTo,
            "min" to Min,
            "max" to Max,
            "+" to Addition,
            "-" to Subtraction,
            "*" to Multiplication,
            "/" to Division,
            "%" to Modulo,

            // logic
            "==" to Equals,
            "!=" to NotEquals,
            "===" to StrictEquals,
            "!==" to NotStrictEquals,
            "!" to Negation,
            "!!" to DoubleNegation,
            "and" to And,
            "or" to Or,
            "if" to If,

            // string
            "cat" to Cat,
            "substr" to Substr,

            // array
            "merge" to Merge,

            // string & array
            "in" to In,
        )
        private val functionalOperations: MutableMap<String, FunctionalLogicOperation> = mutableMapOf(
            // array
            "map" to LogicMap,
            "filter" to Filter,
            "reduce" to Reduce,
            "all" to All,
            "none" to None,
            "some" to Some
        )

        fun addStandardOperation(operationName: String, operation: StandardLogicOperation) = apply {
            if (isNotOperationDuplicate(operationName)) {
                standardOperations[operationName] = operation
            }
        }

        fun addStandardOperations(operations: Map<String, StandardLogicOperation>) = apply {
            operations.forEach { (name, lambda) -> addStandardOperation(name, lambda) }
        }

        fun addFunctionalOperation(operationName: String, operation: FunctionalLogicOperation) = apply {
            if (isNotOperationDuplicate(operationName)) {
                functionalOperations[operationName] = operation
            }
        }

        fun addFunctionalOperations(operations: Map<String, FunctionalLogicOperation>) = apply {
            operations.forEach { (name, lambda) -> addFunctionalOperation(name, lambda) }
        }

        fun addLogger(loggingCallback: ((Any?) -> Unit)) = apply {
            logger = loggingCallback
        }

        fun addStandardConfig(config: StandardLogicOperationConfig) = apply {
            standardLogicOperationConfig = config
        }

        fun addStreamProcessingConfig(config: StreamProcessingConfig) = apply {
            streamProcessingConfig = config
        }

        private fun isNotOperationDuplicate(operationName: String) =
            functionalOperations.contains(operationName).not() && standardOperations.contains(operationName).not()

        fun build(): JsonLogicEngine {
            Log(logger).let { standardOperations.put("log", it) }
            val evaluator = CommonLogicEvaluator(
                standardLogicOperationConfig, LogicOperations(standardOperations, functionalOperations)
            )
            return StreamingJsonLogicEngine(CommonJsonLogicEngine(evaluator), JsonLogicStreamProcessor(streamProcessingConfig))
        }
    }
}

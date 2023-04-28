package io.github.ayushmaanbhav.ruleEngine

import com.fasterxml.jackson.core.type.TypeReference
import com.fasterxml.jackson.databind.ObjectMapper
import io.github.ayushmaanbhav.jsonLogic.JsonLogicEngine
import io.github.ayushmaanbhav.jsonLogic.JsonLogicResult
import io.github.ayushmaanbhav.ruleEngine.config.Config
import io.github.ayushmaanbhav.ruleEngine.exception.RuleEngineException
import io.kotest.assertions.throwables.shouldThrow
import io.mockk.*
import io.mockk.verifySequence
import io.mockk.every
import io.kotest.core.spec.style.BehaviorSpec
import io.kotest.matchers.shouldBe

class JsonLogicEvaluatorTest : BehaviorSpec({
    given("a JsonLogicEvaluator instance 1") {
        val config: Config = mockk()
        val objectMapper: ObjectMapper = mockk()
        val jsonLogic: JsonLogicEngine = mockk()
        every { config.objectMapper } returns objectMapper
        val evaluator = JsonLogicEvaluator(config, jsonLogic)

        val attributes = linkedMapOf<String, Any>("age" to 21, "name" to "John")
        val rule1 = RuleImpl("rule1", """{"===": [{"var": "age"}, 21]}""")
        val rule2 = RuleImpl("rule2", """{"==": [{"var": "name"}, "John"] }""")
        val rules = listOf(rule1, rule2)
        var i = 1
        var j = 1

        every { objectMapper.readValue(any<String>(), any<TypeReference<LinkedHashMap<String, Any>>>()) } returns linkedMapOf("expression" to true)
        every { jsonLogic.evaluate(any<LinkedHashMap<String, Any>>(), any()) } answers { JsonLogicResult.Success(linkedMapOf("output${i++}" to true)) }
        every { objectMapper.convertValue(any(), any<TypeReference<LinkedHashMap<String, Any>>>()) } answers { linkedMapOf("result${j++}" to true) }

        `when`("evaluating rules and attributes") {
            val result = evaluator.evaluate(rules, attributes)

            then("the rules should be evaluated and the output should be returned") {
                result["result1"] shouldBe true
                result["result2"] shouldBe true

                /*verifySequence {
                    objectMapper.readValue(rule1.getExpression(), any<TypeReference<LinkedHashMap<String, Any>>>())
                    jsonLogic.evaluate(any<LinkedHashMap<String, Any>>(), attributes)
                    objectMapper.convertValue(linkedMapOf("output1" to true), any<TypeReference<LinkedHashMap<String, Any>>>())
                    objectMapper.readValue(rule2.getExpression(), any<TypeReference<LinkedHashMap<String, Any>>>())
                    jsonLogic.evaluate(any<LinkedHashMap<String, Any>>(), attributes)
                    objectMapper.convertValue(linkedMapOf("output2" to true), any<TypeReference<LinkedHashMap<String, Any>>>())
                }*/
            }
        }
    }

    given("a JsonLogicEvaluator instance 2") {
        val config: Config = mockk()
        val objectMapper: ObjectMapper = mockk()
        val jsonLogic: JsonLogicEngine = mockk()
        every { config.objectMapper } returns objectMapper
        val evaluator = JsonLogicEvaluator(config, jsonLogic)

        val attributes = linkedMapOf<String, Any>("age" to 21, "name" to "John")
        val rule1 = RuleImpl("rule1", """===": [{"var": "age"}, 21]}""")
        val rule2 = RuleImpl("rule2", """{"==": [{"var": "name"}, "John"] }""")
        val rules = listOf(rule1, rule2)
        var i = 1
        var j = 1

        every { objectMapper.readValue(any<String>(), any<TypeReference<LinkedHashMap<String, Any>>>()) } throws Exception("invalid expression")
        every { jsonLogic.evaluate(any<LinkedHashMap<String, Any>>(), any()) } answers { JsonLogicResult.Success(linkedMapOf("output${i++}" to true)) }
        every { objectMapper.convertValue(any(), any<TypeReference<LinkedHashMap<String, Any>>>()) } answers { linkedMapOf("result${j++}" to true) }

        `when`("evaluating rules and attributes with invalid expression") {
            then("RuleEngineException should be thrown") {
                shouldThrow<RuleEngineException> { evaluator.evaluate(rules, attributes) }
            }
        }
    }

    given("a JsonLogicEvaluator instance 3") {
        val config: Config = mockk()
        val objectMapper: ObjectMapper = mockk()
        val jsonLogic: JsonLogicEngine = mockk()
        every { config.objectMapper } returns objectMapper
        val evaluator = JsonLogicEvaluator(config, jsonLogic)

        val attributes = linkedMapOf<String, Any>("age" to 21, "name" to "John")
        val rule1 = RuleImpl("rule1", """{"===": [{"var": "age"}, 21]}""")
        val rule2 = RuleImpl("rule2", """{"==": [{"var": "name"}, "John"] }""")
        val rules = listOf(rule1, rule2)
        var j = 1

        every { objectMapper.readValue(any<String>(), any<TypeReference<LinkedHashMap<String, Any>>>()) } returns linkedMapOf("expression" to true)
        every { jsonLogic.evaluate(any<LinkedHashMap<String, Any>>(), any()) } throws Exception("invalid json logic")
        every { objectMapper.convertValue(any(), any<TypeReference<LinkedHashMap<String, Any>>>()) } answers { linkedMapOf("result${j++}" to true) }

        `when`("evaluating rules and attributes with invalid expression") {
            then("RuleEngineException should be thrown") {
                shouldThrow<RuleEngineException> { evaluator.evaluate(rules, attributes) }
            }
        }
    }

    given("a JsonLogicEvaluator instance 4") {
        val config: Config = mockk()
        val objectMapper: ObjectMapper = mockk()
        val jsonLogic: JsonLogicEngine = mockk()
        every { config.objectMapper } returns objectMapper
        val evaluator = JsonLogicEvaluator(config, jsonLogic)

        val attributes = linkedMapOf<String, Any>("age" to 21, "name" to "John")
        val rule1 = RuleImpl("rule1", """{"===": [{"var": "age"}, 21]}""")
        val rule2 = RuleImpl("rule2", """{"==": [{"var": "name"}, "John"] }""")
        val rules = listOf(rule1, rule2)
        var i = 1

        every { objectMapper.readValue(any<String>(), any<TypeReference<LinkedHashMap<String, Any>>>()) } returns linkedMapOf("expression" to true)
        every { jsonLogic.evaluate(any<LinkedHashMap<String, Any>>(), any()) } answers { JsonLogicResult.Success(linkedMapOf("output${i++}" to true)) }
        every { objectMapper.convertValue(any(), any<TypeReference<LinkedHashMap<String, Any>>>()) } throws Exception("error converting value")

        `when`("evaluating rules and attributes with invalid expression") {
            then("RuleEngineException should be thrown") {
                shouldThrow<RuleEngineException> { evaluator.evaluate(rules, attributes) }
            }
        }
    }

    given("a JsonLogicEvaluator instance 5") {
        val config: Config = mockk()
        val objectMapper: ObjectMapper = mockk()
        val jsonLogic: JsonLogicEngine = mockk()
        every { config.objectMapper } returns objectMapper
        val evaluator = JsonLogicEvaluator(config, jsonLogic)

        val attributes = linkedMapOf<String, Any>("age" to 21, "name" to "John")
        val rule1 = RuleImpl("rule1", """{"===": [{"var": "age"}, 21]}""")
        val rule2 = RuleImpl("rule2", """{"==": [{"var": "name"}, "John"] }""")
        val rules = listOf(rule1, rule2)

        every { objectMapper.readValue(any<String>(), any<TypeReference<LinkedHashMap<String, Any>>>()) } returns linkedMapOf("expression" to true)
        every { jsonLogic.evaluate(any<LinkedHashMap<String, Any>>(), any()) } answers { JsonLogicResult.Failure.NullResult }

        `when`("evaluating rules and attributes") {
            val result = evaluator.evaluate(rules, attributes)

            then("the rules should be evaluated and the output should be returned") {
                result.isEmpty() shouldBe true
            }
        }
    }

    given("a JsonLogicEvaluator instance 6") {
        val config: Config = mockk()
        val objectMapper: ObjectMapper = mockk()
        val jsonLogic: JsonLogicEngine = mockk()
        every { config.objectMapper } returns objectMapper
        val evaluator = JsonLogicEvaluator(config, jsonLogic)

        val attributes = linkedMapOf<String, Any>("age" to 21, "name" to "John")
        val rule1 = RuleImpl("rule1", """{"===": [{"var": "age"}, 21]}""")
        val rule2 = RuleImpl("rule2", """{"==": [{"var": "name"}, "John"] }""")
        val rules = listOf(rule1, rule2)
        var i = 1

        every { objectMapper.readValue(any<String>(), any<TypeReference<LinkedHashMap<String, Any>>>()) } returns linkedMapOf("expression" to true)
        every { jsonLogic.evaluate(any<LinkedHashMap<String, Any>>(), any()) } answers { JsonLogicResult.Success(linkedMapOf("output${i++}" to true)) }
        every { objectMapper.convertValue(any(), any<TypeReference<LinkedHashMap<String, Any>>>()) } answers { linkedMapOf("result" to true) }

        `when`("evaluating rules and attributes with invalid expression") {
            then("RuleEngineException should be thrown for duplicate result") {
                shouldThrow<RuleEngineException> { evaluator.evaluate(rules, attributes) }
            }
        }
    }

    given("a JsonLogicEvaluator instance 7") {
        val config: Config = mockk()
        val objectMapper: ObjectMapper = mockk()
        val jsonLogic: JsonLogicEngine = mockk()
        every { config.objectMapper } returns objectMapper
        val evaluator = JsonLogicEvaluator(config, jsonLogic)

        val attributes = linkedMapOf<String, Any>("age" to 21, "name" to "John")
        val rule1 = RuleImpl("rule1", """{"===": [{"var": "age"}, 21]}""")
        val rule2 = RuleImpl("rule2", """{"==": [{"var": "name"}, "John"] }""")
        val rules = listOf(rule1, rule2)

        every { objectMapper.readValue(any<String>(), any<TypeReference<LinkedHashMap<String, Any>>>()) } returns linkedMapOf("expression" to true)
        every { jsonLogic.evaluate(any<LinkedHashMap<String, Any>>(), any()) } answers { JsonLogicResult.Failure.MissingOperation }

        `when`("evaluating rules and attributes") {
            then("RuleEngineException should be thrown for failure result") {
                shouldThrow<RuleEngineException> { evaluator.evaluate(rules, attributes) }
            }
        }
    }
})

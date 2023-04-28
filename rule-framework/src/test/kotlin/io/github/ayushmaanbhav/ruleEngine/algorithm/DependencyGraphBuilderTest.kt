package io.github.ayushmaanbhav.ruleEngine.algorithm

import io.github.ayushmaanbhav.ruleEngine.RuleImpl
import io.github.ayushmaanbhav.ruleEngine.exception.GraphContainsCycleException
import io.github.ayushmaanbhav.ruleEngine.exception.MultilpleRulesOutputAttributeException
import io.github.ayushmaanbhav.ruleEngine.model.Query
import io.github.ayushmaanbhav.ruleEngine.model.QueryType
import io.kotest.assertions.throwables.shouldThrow
import io.kotest.core.spec.style.StringSpec
import io.kotest.matchers.collections.shouldContainExactly
import io.kotest.matchers.collections.shouldExistInOrder
import io.kotest.matchers.collections.shouldHaveSize

class DependencyGraphBuilderTest : StringSpec() {
    init {
        "build should create a startNodesByQuery map with correct values" {
            val rule1 = RuleImpl("rule1", "type-1", setOf("attribute-0"), setOf("attribute-1"), setOf("tag-1", "tag-2"))
            val rule2 = RuleImpl("rule2", "type-1", setOf("attribute-1"), setOf("attribute-2"), setOf("tag-1", "tag-2"))
            val rule3 = RuleImpl("rule3", "type-1", setOf("attribute-2"), setOf("attribute-3"), setOf("tag-1", "tag-2"))
            val rule4 = RuleImpl("rule4", "type-1", setOf("attribute-2"), setOf("attribute-4"), setOf("tag-1", "tag-3"))
            val rule5 = RuleImpl("rule5", "type-1", setOf("attribute-1"), setOf("attribute-5"), setOf("tag-1", "tag-3"))
            val rule6 = RuleImpl("rule6", "type-2", setOf("attribute-6"), setOf("attribute-7"), setOf("tag-1", "tag-4"))
            val rule7 = RuleImpl("rule7", "type-1", setOf("attribute-4"), setOf("attribute-8"), setOf("tag-1", "tag-3"))

            val builder = DependencyGraphBuilder<RuleImpl>()
            builder.visit(rule1)
            builder.visit(rule2)
            builder.visit(rule3)
            builder.visit(rule4)
            builder.visit(rule5)
            builder.visit(rule6)
            builder.visit(rule7)
            val graph = builder.build()

            val query1 = Query("type-1", QueryType.RULE_TYPE)
            val query2 = Query("type-2", QueryType.RULE_TYPE)
            val query3 = Query("attribute-8", QueryType.ATTRIBUTE_PATH)
            val query4 = Query("attribute-3", QueryType.ATTRIBUTE_PATH)
            val query5 = Query("attribute-7", QueryType.ATTRIBUTE_PATH)
            val query6 = Query("attribute-5", QueryType.ATTRIBUTE_PATH)
            val query7 = Query("tag-1", QueryType.ATTRIBUTE_TAG)
            val query8 = Query("tag-2", QueryType.ATTRIBUTE_TAG)
            val query9 = Query("tag-3", QueryType.ATTRIBUTE_TAG)
            val query10 = Query("tag-4", QueryType.ATTRIBUTE_TAG)

            var rules = graph.computeExecutableRules(listOf(query1))
            rules.shouldHaveSize(6)
            rules.shouldExistInOrder({ it == rule1 }, { it == rule2 }, { it == rule3 })
            rules.shouldExistInOrder({ it == rule1 }, { it == rule2 }, { it == rule4 }, { it == rule7 })
            rules.shouldExistInOrder({ it == rule1 }, { it == rule5 })

            rules = graph.computeExecutableRules(listOf(query2))
            rules.shouldHaveSize(1)
            rules.shouldContainExactly(rule6)

            rules = graph.computeExecutableRules(listOf(query3))
            rules.shouldHaveSize(4)
            rules.shouldContainExactly(rule1, rule2, rule4, rule7)

            rules = graph.computeExecutableRules(listOf(query4, query5))
            rules.shouldHaveSize(4)
            rules.shouldExistInOrder({ it == rule1 }, { it == rule2 }, { it == rule3 })
            rules.shouldExistInOrder({ it == rule6 })

            rules = graph.computeExecutableRules(listOf(query6, query10))
            rules.shouldHaveSize(3)
            rules.shouldExistInOrder({ it == rule1 }, { it == rule5 })
            rules.shouldExistInOrder({ it == rule6 })

            rules = graph.computeExecutableRules(listOf(query7, query8))
            rules.shouldHaveSize(7)
            rules.shouldExistInOrder({ it == rule1 }, { it == rule2 }, { it == rule3 })
            rules.shouldExistInOrder({ it == rule1 }, { it == rule2 }, { it == rule4 }, { it == rule7 })
            rules.shouldExistInOrder({ it == rule1 }, { it == rule5 })
            rules.shouldExistInOrder({ it == rule6 })

            rules = graph.computeExecutableRules(listOf(query9))
            rules.shouldHaveSize(5)
            rules.shouldExistInOrder({ it == rule1 }, { it == rule2 }, { it == rule4 }, { it == rule7 })
            rules.shouldExistInOrder({ it == rule1 }, { it == rule5 })
        }

        "build should throw MultilpleRulesOutputAttributeException when there is a cycle" {
            val rule1 = RuleImpl("rule1", "type-1", setOf("attribute-0"), setOf("attribute-1"), setOf("tag-1", "tag-2"))
            val rule2 = RuleImpl("rule2", "type-1", setOf("attribute-1"), setOf("attribute-2"), setOf("tag-1", "tag-2"))
            val rule3 = RuleImpl("rule3", "type-1", setOf("attribute-2"), setOf("attribute-3"), setOf("tag-1", "tag-2"))
            val rule4 = RuleImpl("rule4", "type-1", setOf("attribute-2"), setOf("attribute-4"), setOf("tag-1", "tag-3"))
            val rule5 = RuleImpl("rule5", "type-1", setOf("attribute-1"), setOf("attribute-5"), setOf("tag-1", "tag-3"))
            val rule6 = RuleImpl("rule6", "type-2", setOf("attribute-6"), setOf("attribute-7"), setOf("tag-1", "tag-4"))
            val rule7 = RuleImpl("rule7", "type-1", setOf("attribute-4"), setOf("attribute-8"), setOf("tag-1", "tag-3"))
            val rule8 = RuleImpl("rule8", "type-1", setOf("attribute-8"), setOf("attribute-9"), setOf("tag-1", "tag-3"))
            val rule9 = RuleImpl("rule9", "type-1", setOf("attribute-8"), setOf("attribute-1"), setOf("tag-1", "tag-3"))

            val builder = DependencyGraphBuilder<RuleImpl>()
            builder.visit(rule1)
            builder.visit(rule2)
            builder.visit(rule3)
            builder.visit(rule4)
            builder.visit(rule5)
            builder.visit(rule6)
            builder.visit(rule7)
            builder.visit(rule8)
            builder.visit(rule9)

            shouldThrow<MultilpleRulesOutputAttributeException> { builder.build() }
        }

        "build should throw GraphContainsCycleException when there is a cycle" {
            val rule1 = RuleImpl("rule1", "type-1", setOf("attribute-0"), setOf("attribute-1"), setOf("tag-1", "tag-2"))
            val rule2 = RuleImpl("rule2", "type-1", setOf("attribute-1"), setOf("attribute-2"), setOf("tag-1", "tag-2"))
            val rule3 = RuleImpl("rule3", "type-1", setOf("attribute-2"), setOf("attribute-3"), setOf("tag-1", "tag-2"))
            val rule4 = RuleImpl("rule4", "type-1", setOf("attribute-2"), setOf("attribute-4"), setOf("tag-1", "tag-3"))
            val rule5 = RuleImpl("rule5", "type-1", setOf("attribute-1"), setOf("attribute-5"), setOf("tag-1", "tag-3"))
            val rule6 = RuleImpl("rule6", "type-2", setOf("attribute-6"), setOf("attribute-7"), setOf("tag-1", "tag-4"))
            val rule7 = RuleImpl("rule7", "type-1", setOf("attribute-4"), setOf("attribute-8"), setOf("tag-1", "tag-3"))
            val rule8 = RuleImpl("rule8", "type-1", setOf("attribute-8"), setOf("attribute-9"), setOf("tag-1", "tag-3"))
            val rule9 = RuleImpl("rule9", "type-1", setOf("attribute-8"), setOf("attribute-0"), setOf("tag-1", "tag-3"))

            val builder = DependencyGraphBuilder<RuleImpl>()
            builder.visit(rule1)
            builder.visit(rule2)
            builder.visit(rule3)
            builder.visit(rule4)
            builder.visit(rule5)
            builder.visit(rule6)
            builder.visit(rule7)
            builder.visit(rule8)
            builder.visit(rule9)

            shouldThrow<GraphContainsCycleException> { builder.build() }
        }
    }
}
package io.github.ayushmaanbhav.productFarm.config

import io.github.ayushmaanbhav.rule.domain.ruleExpression.OperatorMap.INSTANCE
import io.github.ayushmaanbhav.rule.domain.ruleExpression.operator.Filter
import io.github.ayushmaanbhav.rule.domain.ruleExpression.operator.logical.And
import io.github.ayushmaanbhav.rule.domain.ruleExpression.operator.logical.If
import io.github.ayushmaanbhav.rule.domain.ruleExpression.operator.logical.Not
import io.github.ayushmaanbhav.rule.domain.ruleExpression.operator.logical.Or
import io.github.ayushmaanbhav.rule.domain.ruleExpression.operator.numeric.Equals
import io.github.ayushmaanbhav.rule.domain.ruleExpression.operator.numeric.Greater
import io.github.ayushmaanbhav.rule.domain.ruleExpression.operator.numeric.GreaterEqual
import io.github.ayushmaanbhav.rule.domain.ruleExpression.operator.numeric.NotEqual
import io.github.ayushmaanbhav.rule.domain.ruleExpression.operator.numeric.Smaller
import io.github.ayushmaanbhav.rule.domain.ruleExpression.operator.numeric.SmallerEqual
import io.github.ayushmaanbhav.rule.domain.ruleExpression.operator.numeric.airthmetic.Add
import io.github.ayushmaanbhav.rule.domain.ruleExpression.operator.numeric.airthmetic.Divide
import io.github.ayushmaanbhav.rule.domain.ruleExpression.operator.numeric.airthmetic.Multiply
import io.github.ayushmaanbhav.rule.domain.ruleExpression.operator.numeric.airthmetic.Subtract
import io.github.ayushmaanbhav.rule.domain.ruleExpression.operator.string.Concat
import org.springframework.stereotype.Component

@Component
class RuleFrameworkConfig {
    init {
        val operations = INSTANCE
        operations.registerOperator(And())
        operations.registerOperator(Equals())
        operations.registerOperator(Not())
        operations.registerOperator(Or())
        operations.registerOperator(If())
        operations.registerOperator(Filter())
        operations.registerOperator(Greater())
        operations.registerOperator(Smaller())
        operations.registerOperator(SmallerEqual())
        operations.registerOperator(GreaterEqual())
        operations.registerOperator(NotEqual())
        operations.registerOperator(Add())
        operations.registerOperator(Subtract())
        operations.registerOperator(Multiply())
        operations.registerOperator(Divide())
        operations.registerOperator(Concat())
    }
}

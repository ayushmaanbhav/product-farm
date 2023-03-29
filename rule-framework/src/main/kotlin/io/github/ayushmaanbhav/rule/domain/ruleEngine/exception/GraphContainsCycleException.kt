package io.github.ayushmaanbhav.rule.domain.ruleEngine.exception

import io.github.ayushmaanbhav.common.exception.NonRetryableException

class GraphContainsCycleException(message: String) : NonRetryableException(message)

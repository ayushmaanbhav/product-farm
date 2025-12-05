package io.github.ayushmaanbhav.ruleEngine.exception

import io.github.ayushmaanbhav.common.exception.NonRetryableException

class GraphContainsCycleException(message: String) : NonRetryableException(message)

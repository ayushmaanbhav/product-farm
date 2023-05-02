package io.github.ayushmaanbhav.productFarm.util

import io.github.ayushmaanbhav.productFarm.constant.Constant
import io.github.ayushmaanbhav.productFarm.constant.DisplayNameFormat
import io.github.ayushmaanbhav.productFarm.util.type.DissectedAttributeId
import java.util.*

fun generateUUID(): String = UUID.randomUUID().toString().replace(oldValue = "-", newValue = "")

fun getComponentType(componentType: String) = componentType.convertCaseConvention(CaseType.`lower-dash-case`)

fun getComponentId(componentId: String) = componentId.convertCaseConvention(CaseType.`lower-dash-case`)

fun getAttributeName(name: String) = name.split(Constant.ATTRIBUTE_NAME_SEPARATOR)
    .joinToString(Constant.ATTRIBUTE_NAME_SEPARATOR) { it.convertCaseConvention(CaseType.`lower-dash-case`) }

fun generatePath(productId: String, componentType: String, componentId: String?, name: String): String =
    arrayOf(
        productId,
        getComponentType(componentType),
        componentId?.let { getComponentId(it) },
        getAttributeName(name)
    ).joinToString(Constant.COMPONENT_SEPARATOR)

fun generateHumanDisplayName(componentType: String, componentId: String?, name: String): String =
    arrayOf(
        getComponentType(componentType),
        componentId?.let { getComponentId(it) },
        getAttributeName(name)
    ).filterNotNull().joinToString(Constant.HUMAN_FORMAT_COMPONENT_SEPARATOR)

fun generateOriginalDisplayName(componentType: String, componentId: String?, name: String): String =
    arrayOf(componentType, componentId, name).filterNotNull().joinToString(Constant.ORIGINAL_FORMAT_COMPONENT_SEPARATOR)

fun generateDisplayNames(
    productId: String, componentType: String, componentId: String?, name: String
): List<Pair<DisplayNameFormat, String>> {
    val displayNames = mutableListOf<Pair<DisplayNameFormat, String>>()
    displayNames.add(Pair(DisplayNameFormat.SYSTEM, generatePath(productId, componentType, componentId, name)))
    displayNames.add(Pair(DisplayNameFormat.HUMAN, generateHumanDisplayName(componentType, componentId, name)))
    displayNames.add(Pair(DisplayNameFormat.ORIGINAL, generateOriginalDisplayName(componentType, componentId, name)))
    return displayNames
}

fun dissectAttributeDisplayName(displayName: String): DissectedAttributeId? = displayName
    .split(Constant.ORIGINAL_FORMAT_COMPONENT_SEPARATOR)
    .filterNot(String::isEmpty)
    .takeIf { it.size >= 3 }
    ?.let { DissectedAttributeId(it[0], it[1], it.subList(2, it.size).joinToString(Constant.ORIGINAL_FORMAT_COMPONENT_SEPARATOR)) }

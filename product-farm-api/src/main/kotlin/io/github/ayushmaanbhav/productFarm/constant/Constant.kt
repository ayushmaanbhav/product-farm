package io.github.ayushmaanbhav.productFarm.constant

class Constant {
    companion object {
        // Api messages
        const val CREATED_MESSAGE = "Created"
        const val NOT_FOUND_MESSAGE = "Not Found"
        
        // DB api field constraints
        const val COMPONENT_SEPARATOR = ":"
        const val HUMAN_FORMAT_COMPONENT_SEPARATOR = "."
        const val ORIGINAL_FORMAT_COMPONENT_SEPARATOR = "."
        const val ATTRIBUTE_NAME_SEPARATOR = "."
        val PRODUCT_ID_REGEX = "[a-zA-Z]([_][a-zA-Z0-9]|[a-zA-Z0-9]){0,50}".toRegex()
        val COMPONENT_ID_REGEX = "[a-z]([-][a-z0-9]|[a-z0-9]){0,50}".toRegex()
        val COMPONENT_TYPE_REGEX = "[a-z]([-][a-z]|[a-z]){0,50}".toRegex()
        val ATTRIBUTE_NAME_REGEX = "[a-z]([$ATTRIBUTE_NAME_SEPARATOR][a-z]|[-][a-z0-9]|[a-z0-9]){0,100}".toRegex()
        val ABSTRACT_PATH_REGEX = (PRODUCT_ID_REGEX.pattern
                                   + "[$COMPONENT_SEPARATOR]" + COMPONENT_TYPE_REGEX.pattern
                                   + "([$COMPONENT_SEPARATOR]" + COMPONENT_ID_REGEX.pattern + ")?"
                                   + "[$COMPONENT_SEPARATOR]" + ATTRIBUTE_NAME_REGEX.pattern).toRegex()
        val PATH_REGEX = (PRODUCT_ID_REGEX.pattern
                          + "[$COMPONENT_SEPARATOR]" + COMPONENT_TYPE_REGEX.pattern
                          + "[$COMPONENT_SEPARATOR]" + COMPONENT_ID_REGEX.pattern
                          + "[$COMPONENT_SEPARATOR]" + ATTRIBUTE_NAME_REGEX.pattern).toRegex()
        val DISPLAY_NAME_REGEX = "[a-z]([$ATTRIBUTE_NAME_SEPARATOR][a-z]|[-][a-z0-9]|[a-z0-9]){0,200}".toRegex()
        val ORIGINAL_ATTRIBUTE_NAME_REGEX = "[a-zA-Z]([$ATTRIBUTE_NAME_SEPARATOR][a-zA-Z]|[a-zA-Z0-9]){0,100}".toRegex()
        val ORIGINAL_COMPONENT_TYPE_REGEX = "[a-zA-Z]([_][a-zA-Z]|[a-zA-Z]){0,50}".toRegex()
        val ORIGINAL_COMPONENT_ID_REGEX = "[a-zA-Z]([-_()][a-zA-Z0-9]|[a-zA-Z0-9]){0,50}".toRegex()
        val TAG_REGEX = "[a-z]([-][a-z]|[a-z]){0,50}".toRegex()
        val DATATYPE_REGEX = "[a-z]([-][a-z]|[a-z]){0,50}".toRegex()
        val FUNCTIONALITY_NAME_REGEX = "[a-z]([-][a-z]|[a-z]){0,50}".toRegex()
        val ENUMERATION_NAME_REGEX = "[a-z]([-][a-z]|[a-z]){0,50}".toRegex()
        val ENUMERATION_VALUE_REGEX = "[a-z]([-][a-z]|[a-z]){0,50}".toRegex()
        val DESCRIPTION_REGEX = "[a-zA-Z0-9,.<>/?*()&#;\\-_=+:'\"\\[\\]{}\\s]{0,200}".toRegex()
        val PRODUCT_NAME_REGEX = "[a-zA-Z0-9,.\\-_:']{0,50}".toRegex()
        val RELATIONSHIP_NAME_REGEX = "[a-z]([-][a-z]|[a-z]){0,50}".toRegex()
        val UUID_REGEX = "[a-f0-9]{32}".toRegex()
    }
}

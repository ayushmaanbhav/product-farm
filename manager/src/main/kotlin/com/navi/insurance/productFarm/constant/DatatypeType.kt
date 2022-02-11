package com.navi.insurance.productFarm.constant

enum class DatatypeType {
    OBJECT, ARRAY, INT, NUMBER, BOOLEAN, STRING;
    
    val value = this.name.lowercase().replace("_", "-")
}

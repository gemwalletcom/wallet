package com.gemwallet.android.serializer

import kotlinx.serialization.builtins.serializer
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.JsonTransformingSerializer

object JsonAsStringSerializer: JsonTransformingSerializer<String>(tSerializer = String.serializer()) {
    override fun transformDeserialize(element: JsonElement): JsonElement {
        return JsonPrimitive(value = element.toString())
    }

    override fun transformSerialize(element: JsonElement): JsonElement {
        val content = (element as? JsonPrimitive)?.takeIf { it.isString }?.content ?: return element
        return runCatching { Json.parseToJsonElement(content) }.getOrDefault(element)
    }
}

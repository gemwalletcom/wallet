package com.gemwallet.android.serializer

import com.wallet.core.primitives.StreamEvent
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonTransformingSerializer

object StreamEventSerializer : JsonTransformingSerializer<StreamEvent>(StreamEvent.serializer()) {

    private const val EVENT_KEY = "event"
    private const val TYPE_KEY = "type"

    override fun transformDeserialize(element: JsonElement): JsonElement {
        val obj = element as? JsonObject ?: return element
        if (EVENT_KEY in obj && TYPE_KEY !in obj) {
            return JsonObject(buildMap {
                putAll(obj)
                put(TYPE_KEY, obj.getValue(EVENT_KEY))
                remove(EVENT_KEY)
            })
        }
        return element
    }
}

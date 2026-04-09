package com.gemwallet.android.serializer

import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.TransactionId
import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.JsonDecoder
import kotlinx.serialization.json.JsonEncoder
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import java.io.IOException

object TransactionIdSerializer : KSerializer<TransactionId> {
    override val descriptor: SerialDescriptor =
        PrimitiveSerialDescriptor(TransactionId::class.simpleName!!, PrimitiveKind.STRING)

    override fun serialize(encoder: Encoder, value: TransactionId) = when (encoder) {
        is JsonEncoder -> encoder.encodeJsonElement(JsonPrimitive(value.identifier))
        else -> encoder.encodeString(value.identifier)
    }

    override fun deserialize(decoder: Decoder): TransactionId = when (decoder) {
        is JsonDecoder -> decoder.decodeJsonElement().let {
            try {
                it.jsonPrimitive.contentOrNull?.let(TransactionId::from) ?: throw IOException("TransactionId is null")
            } catch (_: IllegalArgumentException) {
                val jsonObject = it.jsonObject
                val chain = decoder.json.decodeFromJsonElement<Chain>(jsonObject.getValue("chain"))
                val hash = jsonObject.getValue("hash").jsonPrimitive.contentOrNull
                    ?: throw IOException("TransactionId hash is null")
                TransactionId(chain, hash)
            }
        }

        else -> TransactionId.from(decoder.decodeString()) ?: throw IOException("TransactionId is null")
    }
}

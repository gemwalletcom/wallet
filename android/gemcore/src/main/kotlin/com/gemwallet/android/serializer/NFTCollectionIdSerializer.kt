package com.gemwallet.android.serializer

import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toNftCollectionId
import com.wallet.core.primitives.NFTCollectionId
import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.JsonDecoder
import kotlinx.serialization.json.JsonEncoder
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.jsonPrimitive
import java.io.IOException

object NFTCollectionIdSerializer : KSerializer<NFTCollectionId> {
    override val descriptor: SerialDescriptor =
        PrimitiveSerialDescriptor(NFTCollectionId::class.simpleName!!, PrimitiveKind.STRING)

    override fun serialize(encoder: Encoder, value: NFTCollectionId) = when (encoder) {
        is JsonEncoder -> encoder.encodeJsonElement(JsonPrimitive(value.toIdentifier()))
        else -> encoder.encodeString(value.toIdentifier())
    }

    override fun deserialize(decoder: Decoder): NFTCollectionId {
        val raw = when (decoder) {
            is JsonDecoder -> decoder.decodeJsonElement().jsonPrimitive.content
            else -> decoder.decodeString()
        }
        return raw.toNftCollectionId() ?: throw IOException("NFTCollectionId is invalid: $raw")
    }
}

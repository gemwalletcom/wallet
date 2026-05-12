package com.gemwallet.android.serializer

import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toNftAssetId
import com.wallet.core.primitives.NFTAssetId
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

object NFTAssetIdSerializer : KSerializer<NFTAssetId> {
    override val descriptor: SerialDescriptor =
        PrimitiveSerialDescriptor(NFTAssetId::class.simpleName!!, PrimitiveKind.STRING)

    override fun serialize(encoder: Encoder, value: NFTAssetId) = when (encoder) {
        is JsonEncoder -> encoder.encodeJsonElement(JsonPrimitive(value.toIdentifier()))
        else -> encoder.encodeString(value.toIdentifier())
    }

    override fun deserialize(decoder: Decoder): NFTAssetId {
        val raw = when (decoder) {
            is JsonDecoder -> decoder.decodeJsonElement().jsonPrimitive.content
            else -> decoder.decodeString()
        }
        return raw.toNftAssetId() ?: throw IOException("NFTAssetId is invalid: $raw")
    }
}

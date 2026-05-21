package com.gemwallet.android.serializer

import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPerpetualId
import com.wallet.core.primitives.PerpetualId
import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import java.io.IOException

object PerpetualIdSerializer : KSerializer<PerpetualId> {
    override val descriptor: SerialDescriptor =
        PrimitiveSerialDescriptor(PerpetualId::class.simpleName!!, PrimitiveKind.STRING)

    override fun serialize(encoder: Encoder, value: PerpetualId) {
        encoder.encodeString(value.toIdentifier())
    }

    override fun deserialize(decoder: Decoder): PerpetualId {
        val value = decoder.decodeString()
        return value.toPerpetualId() ?: throw IOException("Invalid PerpetualId: $value")
    }
}

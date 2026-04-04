package com.gemwallet.android.serializer

import com.wallet.core.primitives.WalletId
import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder

object WalletIdSerializer : KSerializer<WalletId> {
    override val descriptor: SerialDescriptor =
        PrimitiveSerialDescriptor(WalletId::class.simpleName!!, PrimitiveKind.STRING)

    override fun serialize(encoder: Encoder, value: WalletId) {
        encoder.encodeString(value.id)
    }

    override fun deserialize(decoder: Decoder): WalletId {
        return WalletId(id = decoder.decodeString())
    }
}

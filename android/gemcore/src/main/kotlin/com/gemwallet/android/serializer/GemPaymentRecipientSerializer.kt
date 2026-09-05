package com.gemwallet.android.serializer

import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import uniffi.gemstone.GemPaymentRecipient
import uniffi.gemstone.GemRecipient

object GemPaymentRecipientSerializer : KSerializer<GemPaymentRecipient> {

    @Serializable
    @SerialName("GemPaymentRecipient")
    private data class Fields(
        val recipient: @Serializable(with = GemRecipientSerializer::class) GemRecipient,
        val amount: String? = null,
    )

    override val descriptor: SerialDescriptor = Fields.serializer().descriptor

    override fun serialize(encoder: Encoder, value: GemPaymentRecipient) =
        encoder.encodeSerializableValue(Fields.serializer(), Fields(value.recipient, value.amount))

    override fun deserialize(decoder: Decoder): GemPaymentRecipient =
        decoder.decodeSerializableValue(Fields.serializer()).let { GemPaymentRecipient(it.recipient, it.amount) }
}

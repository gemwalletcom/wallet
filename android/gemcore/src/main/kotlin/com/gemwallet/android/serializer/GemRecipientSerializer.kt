package com.gemwallet.android.serializer

import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import uniffi.gemstone.GemRecipient

object GemRecipientSerializer : KSerializer<GemRecipient> {

    @Serializable
    @SerialName("GemRecipient")
    private data class Fields(
        val address: String,
        val name: String? = null,
        val memo: String? = null,
        val references: List<String> = emptyList(),
    )

    override val descriptor: SerialDescriptor = Fields.serializer().descriptor

    override fun serialize(encoder: Encoder, value: GemRecipient) =
        encoder.encodeSerializableValue(
            Fields.serializer(),
            Fields(value.address, value.name, value.memo, value.references),
        )

    override fun deserialize(decoder: Decoder): GemRecipient =
        decoder.decodeSerializableValue(Fields.serializer())
            .let { GemRecipient(it.address, it.name, it.memo, it.references) }
}

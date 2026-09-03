package com.gemwallet.android.serializer

import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.GemTransferService

object GemPerpetualPositionActionSerializer : KSerializer<GemPerpetualPositionAction> {
    override val descriptor: SerialDescriptor = PrimitiveSerialDescriptor("GemPerpetualPositionAction", PrimitiveKind.STRING)

    override fun serialize(encoder: Encoder, value: GemPerpetualPositionAction) =
        encoder.encodeString(GemTransferService().use { it.encodePositionAction(value) })

    override fun deserialize(decoder: Decoder): GemPerpetualPositionAction =
        GemTransferService().use { it.decodePositionAction(decoder.decodeString()) }
}

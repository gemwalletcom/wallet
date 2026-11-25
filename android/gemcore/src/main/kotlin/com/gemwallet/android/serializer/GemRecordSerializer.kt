package com.gemwallet.android.serializer

import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.modules.SerializersModule
import uniffi.gemstone.GemPaymentRecipient
import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemStakeAmountInput
import uniffi.gemstone.GemTransferData
import uniffi.gemstone.PaymentLink
import uniffi.gemstone.decodeGemPaymentRecipient
import uniffi.gemstone.decodeGemPerpetualPositionAction
import uniffi.gemstone.decodeGemRecipient
import uniffi.gemstone.decodeGemStakeAmountInput
import uniffi.gemstone.decodeGemTransferData
import uniffi.gemstone.decodePaymentLink
import uniffi.gemstone.encodeGemPaymentRecipient
import uniffi.gemstone.encodeGemPerpetualPositionAction
import uniffi.gemstone.encodeGemRecipient
import uniffi.gemstone.encodeGemStakeAmountInput
import uniffi.gemstone.encodeGemTransferData
import uniffi.gemstone.encodePaymentLink
import java.util.Base64

class GemRecordSerializer<T : Any>(name: String, private val encode: (T) -> ByteArray, private val decode: (ByteArray) -> T) : KSerializer<T> {

    override val descriptor: SerialDescriptor = PrimitiveSerialDescriptor(name, PrimitiveKind.STRING)

    override fun serialize(encoder: Encoder, value: T) {
        encoder.encodeString(Base64.getEncoder().encodeToString(encode(value)))
    }

    override fun deserialize(decoder: Decoder): T = decode(Base64.getDecoder().decode(decoder.decodeString()))
}

inline fun <reified T : Any> gemRecordSerializer(noinline encode: (T) -> ByteArray, noinline decode: (ByteArray) -> T): GemRecordSerializer<T> =
    GemRecordSerializer(T::class.java.simpleName, encode, decode)

val gemRecordSerializers = SerializersModule {
    contextual(GemRecipient::class, gemRecordSerializer(::encodeGemRecipient, ::decodeGemRecipient))
    contextual(GemPaymentRecipient::class, gemRecordSerializer(::encodeGemPaymentRecipient, ::decodeGemPaymentRecipient))
    contextual(GemPerpetualPositionAction::class, gemRecordSerializer(::encodeGemPerpetualPositionAction, ::decodeGemPerpetualPositionAction))
    contextual(GemTransferData::class, gemRecordSerializer(::encodeGemTransferData, ::decodeGemTransferData))
    contextual(GemStakeAmountInput::class, gemRecordSerializer(::encodeGemStakeAmountInput, ::decodeGemStakeAmountInput))
    contextual(PaymentLink::class, gemRecordSerializer(::encodePaymentLink, ::decodePaymentLink))
}

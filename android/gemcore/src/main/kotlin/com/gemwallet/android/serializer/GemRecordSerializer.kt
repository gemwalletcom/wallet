package com.gemwallet.android.serializer

import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.modules.SerializersModule
import uniffi.gemstone.FfiConverter
import uniffi.gemstone.FfiConverterTypeGemPaymentRecipient
import uniffi.gemstone.FfiConverterTypeGemPerpetualPositionAction
import uniffi.gemstone.FfiConverterTypeGemRecipient
import uniffi.gemstone.FfiConverterTypeGemTransferData
import uniffi.gemstone.GemPaymentRecipient
import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.GemRecipient
import uniffi.gemstone.GemTransferData
import java.nio.ByteBuffer
import java.util.Base64

class GemRecordSerializer<T : Any>(
    private val converter: FfiConverter<T, *>,
    name: String,
) : KSerializer<T> {

    override val descriptor: SerialDescriptor = PrimitiveSerialDescriptor(name, PrimitiveKind.STRING)

    override fun serialize(encoder: Encoder, value: T) {
        val buffer = ByteBuffer.allocate(converter.allocationSize(value).toInt())
        converter.write(value, buffer)
        encoder.encodeString(Base64.getEncoder().encodeToString(buffer.array().copyOf(buffer.position())))
    }

    override fun deserialize(decoder: Decoder): T {
        val buffer = ByteBuffer.wrap(Base64.getDecoder().decode(decoder.decodeString()))
        val value = converter.read(buffer)
        require(!buffer.hasRemaining()) { "${descriptor.serialName} payload has trailing bytes" }
        return value
    }
}

inline fun <reified T : Any> gemRecordSerializer(converter: FfiConverter<T, *>): GemRecordSerializer<T> =
    GemRecordSerializer(converter, T::class.java.simpleName)

val gemRecordSerializers = SerializersModule {
    contextual(GemRecipient::class, gemRecordSerializer(FfiConverterTypeGemRecipient))
    contextual(GemPaymentRecipient::class, gemRecordSerializer(FfiConverterTypeGemPaymentRecipient))
    contextual(GemPerpetualPositionAction::class, gemRecordSerializer(FfiConverterTypeGemPerpetualPositionAction))
    contextual(GemTransferData::class, gemRecordSerializer(FfiConverterTypeGemTransferData))
}

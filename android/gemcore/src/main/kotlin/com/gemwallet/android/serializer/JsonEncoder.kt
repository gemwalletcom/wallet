package com.gemwallet.android.serializer

import kotlinx.serialization.json.Json
import kotlinx.serialization.modules.SerializersModule
import kotlinx.serialization.modules.contextual
import uniffi.gemstone.GemPaymentRecipient

val jsonEncoder by lazy {
    Json {
        ignoreUnknownKeys = true
        coerceInputValues = true
        explicitNulls = false
        serializersModule = SerializersModule {
            contextual(GemPaymentRecipient::class, GemPaymentRecipientSerializer)
        }
    }
}
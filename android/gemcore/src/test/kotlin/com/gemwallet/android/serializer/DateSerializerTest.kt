package com.gemwallet.android.serializer

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PriceAlert
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import org.junit.Assert.assertEquals
import org.junit.Test

class DateSerializerTest {

    @Test
    fun `serializes dates as iso8601 strings`() {
        val json = jsonEncoder.encodeToString(
            PriceAlert(
                assetId = AssetId(Chain.Bitcoin),
                currency = "USD",
                lastNotifiedAt = 1_775_557_356_209L,
            )
        )

        assertEquals("""{"assetId":"bitcoin","currency":"USD","lastNotifiedAt":"2026-04-07T10:22:36.209Z"}""", json)
    }

    @Test
    fun `deserializes iso8601 strings into epoch millis`() {
        val priceAlert = jsonEncoder.decodeFromString<PriceAlert>(
            """{"assetId":"bitcoin","currency":"USD","lastNotifiedAt":"2026-04-07T10:22:36.209Z"}"""
        )

        assertEquals(1_775_557_356_209L, priceAlert.lastNotifiedAt)
    }
}

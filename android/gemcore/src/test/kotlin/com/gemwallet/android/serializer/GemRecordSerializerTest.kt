package com.gemwallet.android.serializer

import com.gemwallet.android.testkit.mockGemPerpetualTransferData
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemPaymentRecipient
import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.GemRecipient
import java.math.BigInteger

class GemRecordSerializerTest {

    @Test
    fun aRecordRoundTripsThroughTheJsonEncoder() {
        val payment = GemPaymentRecipient(GemRecipient(address = "0x1", name = "Gem", memo = "12345", references = listOf("ref")), amount = "10")

        assertEquals(payment, jsonEncoder.decodeFromString<GemPaymentRecipient>(jsonEncoder.encodeToString(payment)))
    }

    @Test
    fun anEnumWithDataRoundTripsThroughARoutePayload() {
        val action: GemPerpetualPositionAction = GemPerpetualPositionAction.Reduce(mockGemPerpetualTransferData(), BigInteger.TEN)

        assertEquals(action, unpackRoutePayload<GemPerpetualPositionAction>(requireNotNull(action.packRoutePayload())))
    }

    @Test
    fun aCorruptedPayloadDecodesToNothing() {
        assertNull(unpackRoutePayload<GemPaymentRecipient>("invalid"))
    }
}

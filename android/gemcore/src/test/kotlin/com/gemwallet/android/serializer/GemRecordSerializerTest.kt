package com.gemwallet.android.serializer

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockGemPerpetualTransferData
import com.gemwallet.android.testkit.mockHyperCoreBtc
import com.gemwallet.android.testkit.mockHyperCoreUsdc
import com.gemwallet.android.testkit.mockPerpetualPosition
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualProvider
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemPaymentRecipient
import uniffi.gemstone.GemPerpetualPositionAction
import uniffi.gemstone.GemRecipient

class GemRecordSerializerTest {

    @Test
    fun aRecordRoundTripsThroughTheJsonEncoder() {
        val payment = GemPaymentRecipient(GemRecipient(address = "0x1", name = "Gem", memo = "12345", references = listOf("ref")), amount = "10")

        assertEquals(payment, jsonEncoder.decodeFromString<GemPaymentRecipient>(jsonEncoder.encodeToString(payment)))
    }

    @Test
    fun anEnumWithDataRoundTripsThroughARoutePayload() {
        val action: GemPerpetualPositionAction = GemPerpetualPositionAction.Reduce(
            mockGemPerpetualTransferData(
                provider = PerpetualProvider.Hypercore.toGem(),
                direction = PerpetualDirection.Long.toGem(),
                asset = mockHyperCoreBtc().toGem(),
                baseAsset = mockHyperCoreUsdc().toGem(),
                price = 100.0,
                leverage = 1u,
                marginType = PerpetualMarginType.Cross.toGem(),
            ),
            mockPerpetualPosition().toGem(),
        )

        assertEquals(action, unpackRoutePayload<GemPerpetualPositionAction>(requireNotNull(action.packRoutePayload())))
    }

    @Test
    fun aCorruptedPayloadDecodesToNothing() {
        assertNull(unpackRoutePayload<GemPaymentRecipient>("invalid"))
    }
}

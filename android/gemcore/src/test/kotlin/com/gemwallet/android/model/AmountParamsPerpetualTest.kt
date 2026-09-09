package com.gemwallet.android.model

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAssetHyperCoreUBTC
import com.gemwallet.android.testkit.mockGemPerpetualTransferData
import com.gemwallet.android.testkit.mockPerpetualPosition
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemPerpetualPositionAction

class AmountParamsPerpetualTest {

    private val assetId = mockAssetHyperCoreUBTC().id
    private val transferData = mockGemPerpetualTransferData()
    private val perpetualId = PerpetualId(PerpetualProvider.Hypercore, "BTC-PERP")

    private fun perpetual(positionAction: GemPerpetualPositionAction): AmountParams.Perpetual =
        AmountParams.Perpetual(assetId, perpetualId, positionAction)

    @Test
    fun direction_derivesFromPositionActionData() {
        val data = mockGemPerpetualTransferData(direction = PerpetualDirection.Short)
        assertEquals(PerpetualDirection.Short, perpetual(GemPerpetualPositionAction.Open(data)).direction)
    }

    @Test
    fun perpetualParams_surviveTheRoutePayload() {
        val params = perpetual(GemPerpetualPositionAction.Reduce(transferData, mockPerpetualPosition(marginAmount = 1.5).toGem()))

        assertEquals(params, AmountParams.unpack(requireNotNull(params.pack())))
    }
}

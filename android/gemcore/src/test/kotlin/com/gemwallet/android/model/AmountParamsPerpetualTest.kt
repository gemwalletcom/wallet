package com.gemwallet.android.model

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockGemPerpetualTransferData
import com.gemwallet.android.testkit.mockPerpetualId
import com.gemwallet.android.testkit.mockPerpetualPosition
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.PerpetualDirection
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemPerpetualPositionAction

class AmountParamsPerpetualTest {

    @Test
    fun direction_derivesFromPositionActionData() {
        val data = mockGemPerpetualTransferData(direction = PerpetualDirection.Short)
        assertEquals(
            PerpetualDirection.Short,
            AmountParams.Perpetual(
                assetId = mockAssetId(chain = Chain.HyperCore, tokenId = "UBTC::0x8f254b963e8468305d409b33aa137c67::197"),
                perpetualId = mockPerpetualId(symbol = "BTC-PERP"),
                positionAction = GemPerpetualPositionAction.Open(data),
            ).direction,
        )
    }

    @Test
    fun perpetualParams_surviveTheRoutePayload() {
        val params = AmountParams.Perpetual(
            assetId = mockAssetId(chain = Chain.HyperCore, tokenId = "UBTC::0x8f254b963e8468305d409b33aa137c67::197"),
            perpetualId = mockPerpetualId(symbol = "BTC-PERP"),
            positionAction = GemPerpetualPositionAction.Reduce(mockGemPerpetualTransferData(), mockPerpetualPosition(marginAmount = 1.5).toGem()),
        )

        assertEquals(params, AmountParams.unpack(requireNotNull(params.pack())))
    }
}

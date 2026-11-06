package com.gemwallet.android.model

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockGemPerpetualTransferData
import com.gemwallet.android.testkit.mockHyperCoreBtc
import com.gemwallet.android.testkit.mockHyperCoreUsdc
import com.gemwallet.android.testkit.mockPerpetualId
import com.gemwallet.android.testkit.mockPerpetualPosition
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualMarginType
import com.wallet.core.primitives.PerpetualProvider
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemPerpetualPositionAction

class AmountParamsPerpetualTest {

    @Test
    fun perpetualParams_surviveTheRoutePayload() {
        val params = AmountParams.Perpetual(
            assetId = mockHyperCoreBtc().id,
            perpetualId = mockPerpetualId(symbol = "BTC-PERP"),
            positionAction = GemPerpetualPositionAction.Reduce(
                mockGemPerpetualTransferData(
                    provider = PerpetualProvider.Hypercore.toGem(),
                    direction = PerpetualDirection.Long.toGem(),
                    asset = mockHyperCoreBtc().toGem(),
                    baseAsset = mockHyperCoreUsdc().toGem(),
                    price = 100.0,
                    leverage = 1u,
                    marginType = PerpetualMarginType.Cross.toGem(),
                ),
                mockPerpetualPosition(marginAmount = 1.5).toGem(),
            ),
        )

        assertEquals(params, AmountParams.unpack(requireNotNull(params.pack())))
    }
}

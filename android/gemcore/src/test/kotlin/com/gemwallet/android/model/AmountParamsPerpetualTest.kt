package com.gemwallet.android.model

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAmountParamsPerpetual
import com.gemwallet.android.testkit.mockGemPerpetualTransferData
import com.gemwallet.android.testkit.mockPerpetualPosition
import com.wallet.core.primitives.PerpetualDirection
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemPerpetualPositionAction

class AmountParamsPerpetualTest {

    @Test
    fun direction_derivesFromPositionActionData() {
        val data = mockGemPerpetualTransferData(direction = PerpetualDirection.Short)
        assertEquals(PerpetualDirection.Short, mockAmountParamsPerpetual(GemPerpetualPositionAction.Open(data)).direction)
    }

    @Test
    fun perpetualParams_surviveTheRoutePayload() {
        val params = mockAmountParamsPerpetual(GemPerpetualPositionAction.Reduce(mockGemPerpetualTransferData(), mockPerpetualPosition(marginAmount = 1.5).toGem()))

        assertEquals(params, AmountParams.unpack(requireNotNull(params.pack())))
    }
}

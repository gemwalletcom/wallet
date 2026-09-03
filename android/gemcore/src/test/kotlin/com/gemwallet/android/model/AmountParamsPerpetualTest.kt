package com.gemwallet.android.model

import com.gemwallet.android.testkit.mockAssetHyperCoreUBTC
import com.gemwallet.android.testkit.mockGemPerpetualTransferData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.TransactionType
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemPerpetualPositionAction
import java.math.BigInteger

class AmountParamsPerpetualTest {

    private val assetId = mockAssetHyperCoreUBTC().id
    private val transferData = mockGemPerpetualTransferData()
    private val perpetualId = PerpetualId(PerpetualProvider.Hypercore, "BTC-PERP")

    private fun perpetual(positionAction: GemPerpetualPositionAction): AmountParams.Perpetual =
        AmountParams.Perpetual(assetId, perpetualId, positionAction)

    @Test
    fun transactionType_followsThePositionAction() {
        assertEquals(TransactionType.PerpetualOpenPosition, perpetual(GemPerpetualPositionAction.Open(transferData)).transactionType)
        assertEquals(TransactionType.PerpetualModifyPosition, perpetual(GemPerpetualPositionAction.Increase(transferData)).transactionType)
        assertEquals(TransactionType.PerpetualModifyPosition, perpetual(GemPerpetualPositionAction.Reduce(transferData, BigInteger.ZERO)).transactionType)
    }

    @Test
    fun direction_derivesFromPositionActionData() {
        val data = mockGemPerpetualTransferData(direction = PerpetualDirection.Short)
        assertEquals(PerpetualDirection.Short, perpetual(GemPerpetualPositionAction.Open(data)).direction)
    }

    @Test
    fun perpetualParams_surviveTheRoutePayload() {
        val params = perpetual(GemPerpetualPositionAction.Reduce(transferData, BigInteger("1500000")))

        assertEquals(params, AmountParams.unpack(requireNotNull(params.pack())))
    }
}

package com.gemwallet.android.model

import com.gemwallet.android.domains.perpetual.PerpetualPositionAction
import com.gemwallet.android.testkit.mockAssetHyperCoreUBTC
import com.gemwallet.android.testkit.mockPerpetualTransferData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.TransactionType
import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigInteger

class AmountParamsPerpetualTest {

    private val assetId = mockAssetHyperCoreUBTC().id
    private val transferData = mockPerpetualTransferData()

    private fun perpetual(positionAction: PerpetualPositionAction): AmountParams.Perpetual =
        AmountParams.Perpetual(assetId, "BTC-PERP", positionAction)

    @Test
    fun transactionType_isOpenPositionForOpenAction() {
        assertEquals(
            TransactionType.PerpetualOpenPosition,
            perpetual(PerpetualPositionAction.Open(transferData)).transactionType,
        )
    }

    @Test
    fun transactionType_isModifyPositionForIncrease() {
        assertEquals(
            TransactionType.PerpetualModifyPosition,
            perpetual(PerpetualPositionAction.Increase(transferData)).transactionType,
        )
    }

    @Test
    fun transactionType_isModifyPositionForReduce() {
        val reduce = PerpetualPositionAction.Reduce(
            data = transferData,
            available = BigInteger.ZERO,
            positionDirection = PerpetualDirection.Long,
        )
        assertEquals(
            TransactionType.PerpetualModifyPosition,
            perpetual(reduce).transactionType,
        )
    }

    @Test
    fun direction_derivesFromPositionActionData() {
        val data = mockPerpetualTransferData(direction = PerpetualDirection.Short)
        assertEquals(
            PerpetualDirection.Short,
            perpetual(PerpetualPositionAction.Open(data)).direction,
        )
    }
}

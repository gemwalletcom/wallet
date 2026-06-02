package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.coordinators.GetAssetInfo
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetual
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualBalance
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.domains.perpetual.PerpetualPositionAction
import com.gemwallet.android.domains.perpetual.aggregates.PerpetualDetailsDataAggregate
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockPerpetualTransferData
import com.wallet.core.primitives.PerpetualDirection
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class AmountPerpetualProviderTest {

    @Test
    fun `title carries the direction`() {
        val provider = makeProvider(direction = PerpetualDirection.Short)
        val title = provider.title as AmountTitle.Perpetual
        val open = title.action as PerpetualPositionAction.Open
        assertEquals(PerpetualDirection.Short, open.data.direction)
    }

    @Test
    fun `editing then clearing a trigger keeps it cleared when no default is set`() = runTest {
        val provider = makeProvider(scope = backgroundScope)
        provider.setTakeProfit("65000")
        provider.setStopLoss("55000")
        provider.setTakeProfit("")
        provider.setStopLoss(null)
        advanceUntilIdle()
        assertNull(provider.takeProfit.value)
        assertNull(provider.stopLoss.value)
    }

    @Test
    fun `showsAutoclose is true for Open and false for Reduce`() {
        assertTrue(makeProvider().showsAutoclose)
        val reduce = makeProvider(positionAction = PerpetualPositionAction.Reduce(
            data = mockPerpetualTransferData(direction = PerpetualDirection.Long),
            available = BigInteger.TEN,
            positionDirection = PerpetualDirection.Long,
        ))
        assertFalse(reduce.showsAutoclose)
    }

    private fun makeProvider(
        direction: PerpetualDirection = PerpetualDirection.Long,
        positionAction: PerpetualPositionAction = PerpetualPositionAction.Open(
            mockPerpetualTransferData(direction = direction),
        ),
        scope: CoroutineScope = CoroutineScope(Dispatchers.Unconfined + SupervisorJob()),
    ): AmountPerpetualProvider {
        val asset = mockAssetCosmos()
        val getAssetInfo = mockk<GetAssetInfo>(relaxed = true) {
            every { this@mockk.invoke(any()) } returns flowOf(null)
        }
        val userConfig = mockk<UserConfig>(relaxed = true) {
            every { perpetualLeverage() } returns flowOf(5)
            every { perpetualTakeProfit() } returns flowOf(0)
            every { perpetualStopLoss() } returns flowOf(0)
        }
        val perpetualAggregate = mockk<PerpetualDetailsDataAggregate>(relaxed = true)
        val getPerpetual = mockk<GetPerpetual>(relaxed = true) {
            every { getPerpetual(any()) } returns flowOf(perpetualAggregate)
        }
        val getPerpetualBalance = mockk<GetPerpetualBalance>(relaxed = true) {
            every { getBalance() } returns flowOf(null)
        }
        return AmountPerpetualProvider(
            params = AmountParams.Perpetual(asset.id, PerpetualId(PerpetualProvider.Hypercore, "BTC-PERP"), positionAction),
            userConfig = userConfig,
            getAssetInfo = getAssetInfo,
            getPerpetual = getPerpetual,
            getPerpetualBalance = getPerpetualBalance,
            scope = scope,
        )
    }
}

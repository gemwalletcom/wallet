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
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.flowOf
import org.junit.Assert.assertEquals
import org.junit.Test

class AmountPerpetualProviderTest {

    @Test
    fun `setLeverage updates the leverage flow`() {
        val provider = makeProvider()
        provider.setLeverage(10)
        assertEquals(10, provider.leverageState.value?.current)
    }

    @Test
    fun `title carries the direction`() {
        val provider = makeProvider(direction = PerpetualDirection.Short)
        val title = provider.title as AmountTitle.Perpetual
        val open = title.action as PerpetualPositionAction.Open
        assertEquals(PerpetualDirection.Short, open.data.direction)
    }

    private fun makeProvider(direction: PerpetualDirection = PerpetualDirection.Long): AmountPerpetualProvider {
        val asset = mockAssetCosmos()
        val getAssetInfo = mockk<GetAssetInfo>(relaxed = true) {
            every { this@mockk.invoke(any()) } returns flowOf(null)
        }
        val userConfig = mockk<UserConfig>(relaxed = true) {
            every { perpetualLeverage() } returns flowOf(5)
        }
        val perpetualAggregate = mockk<PerpetualDetailsDataAggregate>(relaxed = true) {
            every { maxLeverage } returns 50
        }
        val getPerpetual = mockk<GetPerpetual>(relaxed = true) {
            every { getPerpetual(any()) } returns flowOf(perpetualAggregate)
        }
        val getPerpetualBalance = mockk<GetPerpetualBalance>(relaxed = true) {
            every { getBalance() } returns flowOf(null)
        }
        val positionAction = PerpetualPositionAction.Open(
            mockPerpetualTransferData(direction = direction),
        )
        return AmountPerpetualProvider(
            params = AmountParams.Perpetual(asset.id, "BTC-PERP", positionAction),
            userConfig = userConfig,
            getAssetInfo = getAssetInfo,
            getPerpetual = getPerpetual,
            getPerpetualBalance = getPerpetualBalance,
            scope = CoroutineScope(Dispatchers.Unconfined + SupervisorJob()),
        )
    }
}

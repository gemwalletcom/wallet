package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.coordinators.GetAssetInfo
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetual
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualBalance
import com.gemwallet.android.application.stake.coordinators.GetDelegation
import com.gemwallet.android.application.stake.coordinators.GetDelegations
import com.gemwallet.android.application.stake.coordinators.GetRecommendedValidator
import com.gemwallet.android.application.stake.coordinators.GetStakeValidator
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.domains.perpetual.PerpetualPositionAction
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.DestinationAddress
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockPerpetualTransferData
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.Resource
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.flowOf
import org.junit.Assert.assertTrue
import org.junit.Test

class AmountProviderFactoryTest {

    private val asset = mockAssetCosmos()
    private val factory = AmountProviderFactory(
        transactionBalanceService = mockk(relaxed = true),
        getAssetInfo = mockk<GetAssetInfo>(relaxed = true) {
            every { this@mockk.invoke(any()) } returns flowOf(null)
        },
        getDelegation = mockk<GetDelegation>(relaxed = true) {
            every { this@mockk.invoke(any(), any()) } returns flowOf(null)
        },
        getDelegations = mockk<GetDelegations>(relaxed = true) {
            every { this@mockk.invoke(any(), any()) } returns flowOf(emptyList())
        },
        getRecommendedValidator = mockk<GetRecommendedValidator>(relaxed = true) {
            every { this@mockk.invoke(any()) } returns flowOf(null)
        },
        getStakeValidator = mockk(relaxed = true),
        getPerpetual = mockk<GetPerpetual>(relaxed = true) {
            every { getPerpetual(any()) } returns flowOf(null)
        },
        getPerpetualBalance = mockk<GetPerpetualBalance>(relaxed = true) {
            every { getBalance() } returns flowOf(null)
        },
        userConfig = mockk<UserConfig>(relaxed = true) {
            every { perpetualLeverage() } returns flowOf(5)
        },
    )
    private val scope = CoroutineScope(Dispatchers.Unconfined + SupervisorJob())

    @Test
    fun `Transfer params produce TransferProvider`() {
        val provider = factory.create(
            AmountParams.Transfer(asset.id, DestinationAddress("to", null), null),
            scope,
        )
        assertTrue(provider is AmountTransferProvider)
    }

    @Test
    fun `Stake variants produce StakeProvider`() {
        assertTrue(factory.create(AmountParams.Stake.Delegate(asset.id), scope) is AmountStakeProvider)
        assertTrue(factory.create(AmountParams.Stake.Rewards(asset.id), scope) is AmountStakeProvider)
        assertTrue(factory.create(AmountParams.Stake.Withdraw(asset.id, "v1", "d1"), scope) is AmountStakeProvider)
        assertTrue(factory.create(AmountParams.Stake.Freeze(asset.id, Resource.Bandwidth), scope) is AmountStakeProvider)
        assertTrue(factory.create(AmountParams.Stake.Unfreeze(asset.id, Resource.Bandwidth), scope) is AmountStakeProvider)
    }

    @Test
    fun `Perpetual params produce PerpetualProvider`() {
        val positionAction = PerpetualPositionAction.Open(mockPerpetualTransferData())
        val provider = factory.create(
            AmountParams.Perpetual(asset.id, PerpetualId(PerpetualProvider.Hypercore, "BTC-PERP"), positionAction),
            scope,
        )
        assertTrue(provider is AmountPerpetualProvider)
    }
}

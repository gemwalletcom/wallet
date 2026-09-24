package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.perpetual.cases.GetPerpetual
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.stake.cases.GetDelegation
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.testkit.mockAmountParamsPerpetual
import com.gemwallet.android.testkit.mockAmountParamsTransfer
import com.gemwallet.android.testkit.mockAssetCosmos
import com.wallet.core.primitives.Resource
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemListRow

class AmountProviderFactoryTest {

    private val asset = mockAssetCosmos()
    private val factory = AmountProviderFactory(
        context = mockk(relaxed = true),
        getAssetInfo = mockk<GetAssetInfo>(relaxed = true) {
            every { this@mockk.invoke(any()) } returns flowOf(null)
        },
        getDelegation = mockk<GetDelegation>(relaxed = true) {
            every { this@mockk.invoke(any(), any(), any()) } returns flowOf(null)
        },
        getStakeValidator = mockk(relaxed = true),
        getValidators = mockk<GetValidators>(relaxed = true) {
            every { this@mockk.invoke(any(), any()) } returns flowOf(emptyList())
        },
        getPerpetual = mockk<GetPerpetual>(relaxed = true) {
            every { getPerpetual(any()) } returns flowOf(null)
        },
        getSession = mockk<GetSession>(relaxed = true) {
            every { this@mockk.invoke() } returns MutableStateFlow(null)
        },
        service = mockk<GemAmountServiceInterface>(relaxed = true) {
            every { perpetualAutocloseRow(any(), any()) } returns GemListRow.Loading
        },
        stakeService = mockk(relaxed = true),
    )
    private val scope = CoroutineScope(Dispatchers.Unconfined + SupervisorJob())

    @Test
    fun `Transfer params produce TransferProvider`() {
        val provider = factory.create(mockAmountParamsTransfer(), scope)
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
        val provider = factory.create(mockAmountParamsPerpetual(), scope)
        assertTrue(provider is AmountPerpetualProvider)
    }
}

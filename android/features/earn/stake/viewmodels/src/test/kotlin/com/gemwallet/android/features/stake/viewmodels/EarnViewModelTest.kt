package com.gemwallet.android.features.stake.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.stake.cases.GetDelegations
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.testkit.mockAssetCosmos
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationValidator
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletType
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class EarnViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAssetCosmos()
    private val provider = mockDelegationValidator(id = "earn-provider", apr = 4.0, providerType = StakeProviderType.Earn)
    private val funded = mockDelegation(assetId = asset.id, balance = BigInteger("500"), validator = provider)
    private val empty = mockDelegation(assetId = asset.id, balance = BigInteger.ZERO, delegationId = "empty", validator = provider)

    private val stakeService = mockk<uniffi.gemstone.GemStakeServiceInterface>(relaxed = true) {
        every { earnApr(any(), any()) } returns 4.0
    }
    private val getAssetInfo = mockk<GetAssetInfo> {
        every { this@mockk(asset.id) } returns flowOf(mockAssetInfo(asset = asset))
    }
    private val session = MutableStateFlow(mockSession(wallet = mockWallet(type = WalletType.Multicoin)))
    private val getSession = mockk<GetSession> {
        every { this@mockk() } returns session
    }

    @Before
    fun setUp() = kotlinx.coroutines.Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = kotlinx.coroutines.Dispatchers.resetMain()

    private fun viewModel(
        providers: List<com.wallet.core.primitives.DelegationValidator> = listOf(provider),
        positions: List<com.wallet.core.primitives.Delegation> = listOf(funded, empty),
    ) = EarnViewModel(
        getAssetInfo = getAssetInfo,
        getDelegations = mockk<GetDelegations> {
            every { this@mockk(any(), asset.id, StakeProviderType.Earn) } returns flowOf(positions)
        },
        getValidators = mockk<GetValidators> {
            every { this@mockk(asset.id, StakeProviderType.Earn) } returns flowOf(providers)
        },
        stakeService = stakeService,
        getSession = getSession,
        stateHandle = SavedStateHandle(mapOf(RouteArgument.AssetId.key to asset.id.toIdentifier())),
    )

    @Test
    fun `positions leave out the ones with nothing in them`() = runTest(testDispatcher) {
        val model = viewModel()

        val shown = model.positions.first { it.isNotEmpty() }

        assertEquals(listOf(funded.base.delegationId), shown.map { it.base.delegationId })
    }

    @Test
    fun `the rate is the one core answers for the providers`() = runTest(testDispatcher) {
        val model = viewModel()

        assertEquals(4.0, model.apr.first { it > 0.0 }, 0.0)
    }

    @Test
    fun `depositing needs a provider and a wallet that can sign`() = runTest(testDispatcher) {
        val withProvider = viewModel()

        assertEquals(AmountParams.Earn.Deposit(asset.id, provider.id), withProvider.depositParams.first { it != null })

        val withoutProvider = viewModel(providers = emptyList())
        advanceUntilIdle()
        assertNull("nothing to deposit into", withoutProvider.depositParams.value)

        session.value = mockSession(wallet = mockWallet(type = WalletType.View))
        val watching = viewModel()
        advanceUntilIdle()
        assertNull("a watch-only wallet cannot deposit", watching.depositParams.value)
    }
}

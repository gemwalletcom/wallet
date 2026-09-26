package com.gemwallet.android.features.stake.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetQuery
import com.gemwallet.android.data.services.store.queries.DelegationsQuery
import com.gemwallet.android.data.services.store.queries.ValidatorsQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetData
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationBase
import com.gemwallet.android.testkit.mockDelegationValidator
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletId
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletType
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemDelegationDestination
import uniffi.gemstone.GemEarnInput
import uniffi.gemstone.GemEarnView
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemStakeDelegationItem
import uniffi.gemstone.assetText
import uniffi.gemstone.delegationListRows
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class EarnViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val asset = mockAsset(id = mockAssetId(chain = Chain.Cosmos), name = "Cosmos", symbol = "ATOM", decimals = 6)
    private val provider = mockDelegationValidator(id = "earn-provider", apr = 4.0, providerType = StakeProviderType.Earn)
    private val funded = mockDelegation(base = mockDelegationBase(assetId = asset.id, balance = BigInteger("500"), shares = BigInteger("500")), validator = provider)
    private val empty = mockDelegation(base = mockDelegationBase(assetId = asset.id, balance = BigInteger.ZERO, shares = BigInteger.ZERO, delegationId = "empty"), validator = provider)

    private val aprRow = GemListRow.Text(GemListRowTitle.STAKE_APR, "4.00%")
    private var destination: GemDelegationDestination = GemDelegationDestination.Details
    private val stakeService = mockk<uniffi.gemstone.GemStakeServiceInterface>(relaxed = true) {
        every { earnView(any()) } answers {
            val input = firstArg<GemEarnInput>()
            GemEarnView(
                asset = assetText(input.asset),
                aprRow = aprRow,
                providers = input.providers,
                depositProvider = input.providers.firstOrNull().takeIf { input.walletType != uniffi.gemstone.WalletType.VIEW },
                positions = listOf(GemStakeDelegationItem(funded.toGem(), delegationListRows(listOf(funded.toGem()), input.asset, null, input.currency).first(), destination)),
            )
        }
    }
    private val walletId = mockWalletId()
    private val getCurrentWalletId = mockk<GetCurrentWalletId> {
        every { this@mockk() } returns flowOf(walletId)
    }
    private val assetQuery = mockk<AssetQuery> {
        every { this@mockk(walletId.id, asset.id) } returns flowOf(mockAssetData(asset = asset))
    }
    private val session = MutableStateFlow(mockSession(wallet = mockWallet(type = WalletType.Multicoin)))
    private val getSession = mockk<GetSession> {
        every { this@mockk() } returns session
    }

    @Before
    fun setUp() = kotlinx.coroutines.Dispatchers.setMain(testDispatcher)

    @After
    fun tearDown() = kotlinx.coroutines.Dispatchers.resetMain()

    private fun viewModel(providers: List<com.wallet.core.primitives.DelegationValidator> = listOf(provider), positions: List<com.wallet.core.primitives.Delegation> = listOf(funded, empty)) = EarnViewModel(
        getCurrentWalletId = getCurrentWalletId,
        assetQuery = assetQuery,
        delegationsQuery = mockk<DelegationsQuery> {
            every { this@mockk(any(), asset.id, StakeProviderType.Earn) } returns flowOf(positions)
        },
        validatorsQuery = mockk<ValidatorsQuery> {
            every { this@mockk(asset.id, StakeProviderType.Earn) } returns flowOf(providers)
        },
        stakeService = stakeService,
        getSession = getSession,
        stateHandle = SavedStateHandle(mapOf(RouteArgument.AssetId.key to asset.id.toIdentifier())),
        ioDispatcher = testDispatcher,
        context = mockk(relaxed = true),
    )

    @Test
    fun `positions are the ones core keeps`() = runTest(testDispatcher) {
        val model = viewModel()

        val shown = model.positions.first { it.isNotEmpty() }

        assertEquals(listOf(funded.base.delegationId), shown.map { it.delegation.base.delegationId })
    }

    @Test
    fun `a position opens where core points it`() = runTest(testDispatcher) {
        val model = viewModel()
        model.positions.first { it.isNotEmpty() }
        var opened: Pair<String, String>? = null

        model.onPosition(funded, onOpenDetail = { validator, delegation -> opened = validator to delegation }, onAmount = {}, onConfirm = {})

        assertEquals(funded.validator.id to funded.base.delegationId, opened)
    }

    @Test
    fun `the rate row is the one core answers for the providers`() = runTest(testDispatcher) {
        val model = viewModel()

        assertEquals(aprRow, model.aprRow.first { it == aprRow })
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

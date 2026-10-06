package com.gemwallet.android.features.stake.viewmodels

import android.net.Uri
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetQuery
import com.gemwallet.android.data.services.store.queries.DelegationsQuery
import com.gemwallet.android.data.services.store.queries.ValidatorsQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetData
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockDelegation
import com.gemwallet.android.testkit.mockDelegationBase
import com.gemwallet.android.testkit.mockDelegationValidator
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWalletId
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.StakeProviderType
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import io.mockk.verify
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemStakeActionKind
import uniffi.gemstone.GemStakeAmountInput
import uniffi.gemstone.GemStakeDestination
import uniffi.gemstone.GemStakeServiceInterface
import uniffi.gemstone.GemStakeViewState
import uniffi.gemstone.Resource
import uniffi.gemstone.assetText
import java.math.BigInteger
import kotlin.time.Duration.Companion.seconds

@OptIn(ExperimentalCoroutinesApi::class)
class StakeViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(testDispatcher)

    private val asset = mockAsset(id = mockAssetId(chain = Chain.Cosmos), name = "Cosmos", symbol = "ATOM", decimals = 6u)
    private val delegation = mockDelegation(base = mockDelegationBase(assetId = asset.id, balance = BigInteger("77"), shares = BigInteger("77")), validator = mockDelegationValidator(chain = asset.id.chain))

    private val walletId = mockWalletId()
    private val getCurrentWalletId = mockk<GetCurrentWalletId> {
        every { this@mockk() } returns flowOf(walletId)
    }
    private val assetQuery = mockk<AssetQuery> {
        every { this@mockk(walletId.id, asset.id) } returns flowOf(mockAssetData(asset = asset))
    }
    private val getWalletAssets = mockk<GetWalletAssets> {
        every { this@mockk() } returns MutableStateFlow(emptyList())
    }
    private val delegationsQuery = mockk<DelegationsQuery> {
        every { this@mockk(any(), asset.id, StakeProviderType.Stake) } returns flowOf(listOf(delegation))
    }
    private val validatorsQuery = mockk<ValidatorsQuery> {
        every { this@mockk(asset.id, StakeProviderType.Stake) } returns flowOf(emptyList())
    }
    private val getSession = mockk<GetSession> {
        every { this@mockk() } returns MutableStateFlow(mockSession())
    }
    private val stakeService = mockk<GemStakeServiceInterface>(relaxed = true) {
        every { isAvailable() } returns true
        every { stakeViewState(any()) } returns GemStakeViewState(
            asset = assetText(asset.toGem()),
            sections = emptyList(),
            infoRows = emptyList(),
            actions = emptyList(),
            resourceRows = emptyList(),
            delegations = emptyList(),
            delegationsPhase = null,
            docsUrl = null,
        )
    }

    @Before
    fun setUp() {
        mockkStatic(Uri::class)
        every { Uri.parse(any()) } returns mockk(relaxed = true)
    }

    @After
    fun tearDown() {
        unmockkStatic(Uri::class)
    }

    @Test
    fun `unavailable staking shows the sheet and keeps its amount screen closed`() = runTest(testDispatcher) {
        every { stakeService.isAvailable() } returns false
        val viewModel = createViewModel()
        val destination = GemStakeDestination.Amount(GemStakeAmountInput.Stake(delegation.validator.toGem()))
        val amount: AmountTransactionAction = mockk(relaxed = true)
        val confirm: ConfirmTransactionAction = mockk(relaxed = true)
        try {
            viewModel.onSelect(GemStakeActionKind.STAKE, destination, amount, confirm)
            assertEquals(GemInfoTopic.RegionUnavailable, viewModel.infoSheet.value)
            verify(exactly = 0) { amount(any()) }
            verify(exactly = 0) { confirm(any()) }

            viewModel.infoSheet.value = null
            every { stakeService.isAvailable() } returns true
            viewModel.onSelect(GemStakeActionKind.STAKE, destination, amount, confirm)
            assertNull(viewModel.infoSheet.value)
            verify(exactly = 1) { amount(AmountParams.Stake(asset.id, destination.input)) }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `unavailable staking keeps freeze unfreeze and claiming available`() = runTest(testDispatcher) {
        every { stakeService.isAvailable() } returns false
        val viewModel = createViewModel()
        val freeze = GemStakeAmountInput.Freeze(Resource.ENERGY)
        val unfreeze = GemStakeAmountInput.Unfreeze(Resource.BANDWIDTH)
        val transfer = mockGemTransferData()
        val amount: AmountTransactionAction = mockk(relaxed = true)
        val confirm: ConfirmTransactionAction = mockk(relaxed = true)
        try {
            viewModel.onSelect(GemStakeActionKind.FREEZE, GemStakeDestination.Amount(freeze), amount, confirm)
            viewModel.onSelect(GemStakeActionKind.UNFREEZE, GemStakeDestination.Amount(unfreeze), amount, confirm)
            viewModel.onSelect(GemStakeActionKind.CLAIM_REWARDS, GemStakeDestination.Confirm(transfer), amount, confirm)
            assertNull(viewModel.infoSheet.value)
            verify(exactly = 0) { stakeService.isAvailable() }
            verify(exactly = 1) { amount(AmountParams.Stake(asset.id, freeze)) }
            verify(exactly = 1) { amount(AmountParams.Stake(asset.id, unfreeze)) }
            verify(exactly = 1) { confirm(match { it.data == transfer }) }
        } finally {
            viewModel.viewModelScope.cancel()
        }
    }

    @Test
    fun `a failed delegations sync stops the spinner instead of taking the screen down`() = runTest(testDispatcher, timeout = 10.seconds) {
        val offline = GemServiceException.Gateway("Network offline")
        coEvery { stakeService.refresh(asset.id.chain.string, any()) } returns GemLoadState.Error(offline)

        val viewModel = createViewModel()

        assertEquals(listOf(true, false), viewModel.isSync.take(2).toList())

        runCurrent()
        assertEquals(listOf(delegation), viewModel.delegations.value)
        verify { stakeService.stakeViewState(match { it.state == GemLoadState.Error(offline) }) }
    }

    private fun createViewModel() = StakeViewModel(
        getCurrentWalletId = getCurrentWalletId,
        assetQuery = assetQuery,
        getWalletAssets = getWalletAssets,
        delegationsQuery = delegationsQuery,
        validatorsQuery = validatorsQuery,
        service = stakeService,
        getSession = getSession,
        stateHandle = SavedStateHandle(mapOf(RouteArgument.AssetId.key to asset.id.toIdentifier())),
        ioDispatcher = testDispatcher,
    )
}

package com.gemwallet.android.features.assets.viewmodels

import com.gemwallet.android.application.assets.coordinators.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.coordinators.GetHideBalancesState
import com.gemwallet.android.application.assets.coordinators.GetImportInProgress
import com.gemwallet.android.application.assets.coordinators.GetShowWelcomeBanner
import com.gemwallet.android.application.assets.coordinators.GetWalletSummary
import com.gemwallet.android.application.assets.coordinators.HideAsset
import com.gemwallet.android.application.assets.coordinators.HideWelcomeBanner
import com.gemwallet.android.application.assets.coordinators.SyncAssets
import com.gemwallet.android.application.assets.coordinators.ToggleAssetPin
import com.gemwallet.android.application.assets.coordinators.ToggleHideBalances
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.testkit.mockAsset
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class AssetsViewModelTest {

    private val testDispatcher = StandardTestDispatcher()
    private val activeAssetsFlow = MutableStateFlow(
        listOf(
            assetAggregate(chain = Chain.Solana, symbol = "SOL", pinned = true),
            assetAggregate(chain = Chain.Ethereum, symbol = "ETH", pinned = false),
        )
    )

    private val syncAssets = mockk<SyncAssets>(relaxed = true)
    private val hideAsset = mockk<HideAsset>(relaxed = true)
    private val toggleAssetPin = mockk<ToggleAssetPin>(relaxed = true)
    private val toggleHideBalances = mockk<ToggleHideBalances>(relaxed = true)
    private val hideWelcomeBanner = mockk<HideWelcomeBanner>(relaxed = true)
    private val getImportInProgress = object : GetImportInProgress {
        override fun invoke(): Flow<Boolean> = flowOf(false)
    }
    private val getActiveAssetsInfo = object : GetActiveAssetsInfo {
        override fun getAssetsInfo(hideBalance: Boolean): Flow<List<AssetInfoDataAggregate>> = activeAssetsFlow
    }
    private val getWalletSummary = mockk<GetWalletSummary>(relaxed = true) {
        every { getWalletSummary() } returns flowOf(null)
    }
    private val getHideBalancesState = object : GetHideBalancesState {
        override fun invoke(): Flow<Boolean> = flowOf(false)
    }
    private val getShowWelcomeBanner = object : GetShowWelcomeBanner {
        override fun invoke(isWalletEmpty: Flow<Boolean>): Flow<Boolean> {
            return isWalletEmpty.combine(flowOf(true)) { isEmpty, _ -> isEmpty }
        }
    }

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `pinned and unpinned assets replay current wallet assets`() = runTest(testDispatcher) {
        val viewModel = createViewModel()

        advanceUntilIdle()

        assertEquals(listOf(activeAssetsFlow.value[0]), viewModel.pinnedAssets.value)
        assertEquals(listOf(activeAssetsFlow.value[1]), viewModel.unpinnedAssets.value)
    }

    @Test
    fun `show welcome banner stays true for created wallet with no assets`() = runTest(testDispatcher) {
        activeAssetsFlow.value = emptyList()

        val viewModel = createViewModel()
        advanceUntilIdle()

        assertTrue(viewModel.showWelcomeBanner.value)
    }

    private fun createViewModel() = AssetsViewModel(
        syncAssets = syncAssets,
        hideAsset = hideAsset,
        toggleAssetPin = toggleAssetPin,
        toggleHideBalances = toggleHideBalances,
        hideWelcomeBanner = hideWelcomeBanner,
        getImportInProgress = getImportInProgress,
        getActiveAssetsInfo = getActiveAssetsInfo,
        getWalletSummary = getWalletSummary,
        getHideBalancesState = getHideBalancesState,
        getShowWelcomeBanner = getShowWelcomeBanner,
    )

    private fun assetAggregate(
        chain: Chain,
        symbol: String,
        pinned: Boolean,
    ): AssetInfoDataAggregate {
        val asset = mockAsset(chain = chain, name = symbol, symbol = symbol)
        return AssetInfoDataAggregate(
            id = asset.id,
            asset = asset,
            title = asset.name,
            balance = "1.0 $symbol",
            balanceEquivalent = "$1.00",
            isZeroBalance = false,
            price = null,
            position = 0,
            pinned = pinned,
            accountAddress = "address-$symbol",
        )
    }
}

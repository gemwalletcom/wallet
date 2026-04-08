package com.gemwallet.android.features.assets.viewmodels

import com.gemwallet.android.application.assets.coordinators.GetActiveAssetsInfo
import com.gemwallet.android.application.assets.coordinators.GetWalletSummary
import com.gemwallet.android.application.wallet_import.coordinators.GetImportWalletState
import com.gemwallet.android.application.wallet_import.values.ImportWalletState
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.WalletSource
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.Flow
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
    private val wallet = mockWallet(
        id = "wallet-id",
        accounts = listOf(mockAccount(Chain.Solana)),
        source = WalletSource.Create,
    )
    private val sessionFlow = MutableStateFlow<Session?>(Session(wallet, Currency.USD))
    private val hideBalancesFlow = MutableStateFlow(false)
    private val welcomeBannerHiddenFlow = MutableStateFlow(false)
    private val importStateFlow = MutableStateFlow(ImportWalletState.Complete)
    private val activeAssetsFlow = MutableStateFlow(
        listOf(
            assetAggregate(chain = Chain.Solana, symbol = "SOL", pinned = true),
            assetAggregate(chain = Chain.Ethereum, symbol = "ETH", pinned = false),
        )
    )

    private val sessionRepository = mockk<SessionRepository>(relaxed = true) {
        every { session() } returns sessionFlow
    }
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)
    private val userConfig = mockk<UserConfig>(relaxed = true) {
        every { isHideBalances() } returns hideBalancesFlow
        every { isWelcomeBannerHidden(wallet.id) } returns welcomeBannerHiddenFlow
    }
    private val getImportWalletState = mockk<GetImportWalletState>(relaxed = true) {
        every { getImportState(wallet.id) } returns importStateFlow
    }
    private val getActiveAssetsInfo = object : GetActiveAssetsInfo {
        override fun getAssetsInfo(hideBalance: Boolean): Flow<List<AssetInfoDataAggregate>> = activeAssetsFlow
    }
    private val getWalletSummary = mockk<GetWalletSummary>(relaxed = true) {
        every { getWalletSummary() } returns flowOf(null)
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
        sessionRepository = sessionRepository,
        assetsRepository = assetsRepository,
        userConfig = userConfig,
        getImportWalletState = getImportWalletState,
        getActiveAssetsInfo = getActiveAssetsInfo,
        getWalletSummary = getWalletSummary,
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

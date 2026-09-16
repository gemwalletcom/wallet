package com.gemwallet.android.features.asset_select.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectSearch
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.models.AssetToast
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAssetAction
import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemSelectAssetFlow
import uniffi.gemstone.GemSelectAssetScope
import uniffi.gemstone.GemSelectAssetType

@OptIn(ExperimentalCoroutinesApi::class)
class BaseAssetSelectViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<BaseAssetSelectViewModel>()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
        Dispatchers.resetMain()
    }

    private val wallet = mockWallet(
        id = "multicoin_0xabc",
        accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xabc"), mockAccount(chain = Chain.Bitcoin, address = "bc1q")),
    )

    private val ethereum = mockAsset(chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH")
    private val bitcoin = mockAsset(chain = Chain.Bitcoin, name = "Bitcoin", symbol = "BTC")

    private fun sendFlow(): GemSelectAssetFlow = mockk(relaxed = true) {
        every { action } returns GemAssetAction.SEND
        every { scope } returns GemSelectAssetScope.WALLET
        every { filters } returns emptyList()
        every { popularSection } returns false
        every { recents } returns true
        every { networkSearch } returns false
        every { enablesPriceAlert } returns false
    }

    private fun viewModel(
        items: List<AssetInfo>,
        recents: RecentAssetsService = mockk(relaxed = true) {
            every { getRecentAssets(any()) } returns flowOf(emptyList())
        },
        service: GemAssetSelectionServiceInterface = mockk(relaxed = true) {
            every { flow(any()) } returns sendFlow()
            every { searchDebounceMilliseconds() } returns 0u
            every { filterChains(any()) } returns emptyList()
        },
    ): BaseAssetSelectViewModel {
        val session: GetSession = mockk {
            every { this@mockk.invoke() } returns MutableStateFlow(Session(wallet = wallet, currency = Currency.USD))
        }
        val search = object : SelectSearch {
            override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> = flowOf(items)
        }
        return BaseAssetSelectViewModel(session, recents, service, search, GemSelectAssetType.Send)
            .also { models.add(it) }
    }

    @Test
    fun `the chain filter narrows the list and toggling it back restores it`() = runTest(dispatcher) {
        val model = viewModel(listOf(mockAssetInfo(asset = ethereum), mockAssetInfo(asset = bitcoin)))

        assertEquals(2, model.unpinned.first { it.size == 2 }.size)

        model.onChainFilter(Chain.Ethereum)
        assertEquals(listOf("Ethereum"), model.unpinned.first { it.size == 1 }.map { it.asset.name })

        model.onChainFilter(Chain.Ethereum)
        assertEquals(2, model.unpinned.first { it.size == 2 }.size)
    }

    @Test
    fun `clearing the filters puts every chain back`() = runTest(dispatcher) {
        val model = viewModel(listOf(mockAssetInfo(asset = ethereum), mockAssetInfo(asset = bitcoin)))
        model.unpinned.first { it.size == 2 }

        model.setChainFilter(listOf(Chain.Bitcoin))
        assertEquals(listOf("Bitcoin"), model.unpinned.first { it.size == 1 }.map { it.asset.name })

        model.onBalanceFilter(true)
        assertTrue(model.balanceFilter.value)

        model.onClearFilters()
        assertEquals(emptyList<Chain>(), model.chainFilter.value)
        assertTrue(!model.balanceFilter.value)
        assertEquals(2, model.unpinned.first { it.size == 2 }.size)
    }

    @Test
    fun `pinning an asset tells Core and names it in the toast`() = runTest(dispatcher) {
        val service: GemAssetSelectionServiceInterface = mockk(relaxed = true) {
            every { flow(any()) } returns sendFlow()
            every { searchDebounceMilliseconds() } returns 0u
            every { filterChains(any()) } returns emptyList()
        }
        val model = viewModel(listOf(mockAssetInfo(asset = ethereum)), service = service)
        model.unpinned.first { it.isNotEmpty() }

        val toast = CompletableDeferred<AssetToast>()
        val collector = launch { toast.complete(model.toastEvents.first()) }

        model.onTogglePin(ethereum.id)

        assertEquals(AssetToast.Pin("Ethereum", true), toast.await())
        coVerify { service.setAssetPinned(any(), true) }
        collector.cancel()
    }
}

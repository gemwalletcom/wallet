package com.gemwallet.android.features.asset_select.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.values.AssetsQueryFilter
import com.gemwallet.android.application.assets.values.chains
import com.gemwallet.android.application.assets.values.toQueryFilter
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.RecentActivityQuery
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectAssetFilters
import com.gemwallet.android.features.asset_select.viewmodels.models.SelectSearch
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ToastMessage
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
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
import uniffi.gemstone.GemAssetFilter
import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemSelectAssetFlow
import uniffi.gemstone.GemSelectAssetScope
import uniffi.gemstone.GemSelectAssetType
import uniffi.gemstone.GemSelectAssetWalletFlow

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
        id = WalletId("multicoin_0xabc"),
        accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xabc"), mockAccount(chain = Chain.Bitcoin, address = "bc1q")),
    )

    private val ethereum = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18)
    private val bitcoin = mockAsset(name = "Bitcoin", symbol = "BTC", decimals = 8)

    private fun sendFlow(): GemSelectAssetFlow = mockk(relaxed = true) {
        every { action } returns GemAssetAction.SEND
        every { scope } returns GemSelectAssetScope.WALLET
        every { filters } returns emptyList()
        every { popularSection } returns false
        every { recents } returns true
        every { networkSearch } returns false
        every { enablesPriceAlert } returns false
        every { appliedFilters(any(), any()) } answers {
            val chains = firstArg<List<String>>()
            listOfNotNull(GemAssetFilter.Chains(chains).takeIf { chains.isNotEmpty() }, GemAssetFilter.HasBalance.takeIf { secondArg<Boolean>() })
        }
    }

    private fun viewModel(
        items: List<AssetInfo>,
        recents: RecentActivityQuery = mockk(relaxed = true) {
            every { this@mockk(any(), any(), any(), any()) } returns flowOf(emptyList())
        },
        service: GemAssetSelectionServiceInterface = mockk(relaxed = true) {
            every { flow(any()) } returns sendFlow()
            every { walletFlow(any(), any()) } answers { GemSelectAssetWalletFlow(flow = firstArg<GemSelectAssetType>().flow(), chains = emptyList(), showsAddToken = false, showsChainFilter = false) }
        },
    ): BaseAssetSelectViewModel {
        val session: GetSession = mockk {
            every { this@mockk.invoke() } returns MutableStateFlow(mockSession(wallet = wallet))
        }
        val search = object : SelectSearch {
            override fun items(filters: Flow<SelectAssetFilters?>): Flow<List<AssetInfo>> = filters.filterNotNull().map { current ->
                val query = current.queryFilters()
                val chains = query.map { it.toQueryFilter() }.chains()
                items.filter { (chains.isEmpty() || it.asset.id.chain in chains) && (GemAssetFilter.HasBalance !in query || it.balance.balance.available.signum() > 0) }.take(current.limit)
            }
        }
        return BaseAssetSelectViewModel(session, recents, service, search, GemSelectAssetType.Send, dispatcher, mockk(relaxed = true))
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
    fun `the picker lists at most a hundred assets`() = runTest(dispatcher) {
        val model = viewModel((1..101).map { mockAssetInfo(asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum, tokenId = "0x$it"), type = AssetType.ERC20)) })

        assertEquals(100, model.unpinned.first { it.isNotEmpty() }.size)
    }

    @Test
    fun `recents follow the chain filter`() = runTest(dispatcher) {
        val requests = MutableStateFlow<List<Set<AssetsQueryFilter>>>(emptyList())
        val recents: RecentActivityQuery = mockk(relaxed = true) {
            every { this@mockk(wallet.id, any(), any(), any()) } answers {
                requests.value += listOf(thirdArg<Set<AssetsQueryFilter>>())
                flowOf(emptyList())
            }
        }
        val model = viewModel(emptyList(), recents = recents)

        model.setChainFilter(listOf(Chain.Bitcoin))

        val bitcoin = setOf(AssetsQueryFilter.Chains(listOf(Chain.Bitcoin)))
        assertEquals(bitcoin, requests.first { it.lastOrNull() == bitcoin }.last())
        assertEquals(setOf(GemAssetFilter.Chains(listOf(Chain.Bitcoin.string))), model.assetFilters())
    }

    @Test
    fun `pinning an asset tells Core and names it in the toast`() = runTest(dispatcher) {
        val service: GemAssetSelectionServiceInterface = mockk(relaxed = true) {
            every { flow(any()) } returns sendFlow()
            every { walletFlow(any(), any()) } answers { GemSelectAssetWalletFlow(flow = firstArg<GemSelectAssetType>().flow(), chains = emptyList(), showsAddToken = false, showsChainFilter = false) }
        }
        val model = viewModel(listOf(mockAssetInfo(asset = ethereum)), service = service)
        model.unpinned.first { it.isNotEmpty() }

        val toast = CompletableDeferred<ToastMessage>()
        val collector = launch { toast.complete(model.toastEvents.first()) }

        model.onTogglePin(ethereum.id)

        assertEquals(R.drawable.ic_push_pin, toast.await().image)
        coVerify { service.setAssetPinned(any(), true) }
        collector.cancel()
    }
}

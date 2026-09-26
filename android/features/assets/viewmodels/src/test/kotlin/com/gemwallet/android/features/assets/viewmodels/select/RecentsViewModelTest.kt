package com.gemwallet.android.features.assets.viewmodels.select

import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.RecentActivityQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.assets.viewmodels.select.models.RecentsUIState
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.RecentActivityType
import com.wallet.core.primitives.RecentAsset
import com.wallet.core.primitives.WalletId
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.collections.immutable.persistentListOf
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.coroutines.withContext
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAssetFilter
import uniffi.gemstone.GemEmptyStateKind
import uniffi.gemstone.GemRecentActivityService
import uniffi.gemstone.GemRecentsCounts
import uniffi.gemstone.GemRecentsViewState

@OptIn(ExperimentalCoroutinesApi::class)
class RecentsViewModelTest {

    private val testDispatcher = UnconfinedTestDispatcher()

    private val solAsset = mockAsset(id = mockAssetId(chain = Chain.Solana), name = "Solana", symbol = "SOL", decimals = 9)
    private val ethAsset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18)
    private val recentItems = listOf(
        RecentAsset(asset = solAsset, createdAt = 1000L),
        RecentAsset(asset = ethAsset, createdAt = 2000L),
    )

    private val getCurrentWalletId = object : GetCurrentWalletId {
        override fun invoke(): Flow<WalletId> = flowOf(WalletId("wallet-1"))
    }
    private val recentActivityQuery = mockk<RecentActivityQuery>(relaxed = true)
    private val recentActivityService = mockk<GemRecentActivityService>(relaxed = true) {
        every { viewState(any(), any()) } answers {
            val assets = firstArg<List<uniffi.gemstone.Asset>>()
            GemRecentsViewState(
                matchingAssetIds = assets.map { it.id },
                sections = GemRecentsCounts(assets.size.toUInt(), assets.size.toUInt()).sections(false),
            )
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
    fun `show makes visible and dismiss hides`() = runTest(testDispatcher) {
        val vm = RecentsViewModel(getCurrentWalletId, recentActivityQuery, recentActivityService, testDispatcher)

        assertFalse(vm.visible.value)

        vm.show(filters = setOf(GemAssetFilter.HasBalance))
        advanceUntilIdle()
        assertTrue(vm.visible.value)

        vm.dismiss()
        advanceUntilIdle()
        assertFalse(vm.visible.value)
    }

    @Test
    fun `uiModel keeps content after dismiss`() = runTest(testDispatcher) {
        every { recentActivityQuery(WalletId("wallet-1"), any(), any(), 0) } returns flowOf(recentItems)
        val vm = RecentsViewModel(getCurrentWalletId, recentActivityQuery, recentActivityService, testDispatcher)

        vm.show()
        vm.uiModel.first { it.items.isNotEmpty() }

        vm.dismiss()
        withContext(Dispatchers.Default) { delay(100) }
        assertEquals(recentItems, vm.uiModel.value.items)
    }

    @Test
    fun `clear delegates to coordinator with current types`() = runTest(testDispatcher) {
        val vm = RecentsViewModel(getCurrentWalletId, recentActivityQuery, recentActivityService, testDispatcher)
        val types = listOf(RecentActivityType.Swap)
        vm.show(types = types)
        advanceUntilIdle()

        vm.onClear()
        advanceUntilIdle()

        coVerify { recentActivityService.clear(types.map { it.toGem() }) }
    }

    @Test
    fun `uiModel properties derive correctly`() {
        val withItems = RecentsUIState(
            items = recentItems.toImmutableList(),
            sections = GemRecentsCounts(recents = 5u, matching = 5u).sections(false),
        )
        assertFalse(withItems.isEmpty)
        assertTrue(withItems.showClear)
        assertNull(withItems.emptyState)

        val searchNoResults = RecentsUIState(
            items = persistentListOf(),
            sections = GemRecentsCounts(recents = 5u, matching = 0u).sections(true),
        )
        assertTrue(searchNoResults.isEmpty)
        assertFalse(searchNoResults.showClear)
        assertEquals(GemEmptyStateKind.SEARCH_ASSETS, searchNoResults.emptyState)

        val noRecents = RecentsUIState(
            items = persistentListOf(),
            sections = GemRecentsCounts(recents = 0u, matching = 0u).sections(false),
        )
        assertTrue(noRecents.isEmpty)
        assertFalse(noRecents.showClear)
        assertEquals(GemEmptyStateKind.RECENTS, noRecents.emptyState)
    }
}

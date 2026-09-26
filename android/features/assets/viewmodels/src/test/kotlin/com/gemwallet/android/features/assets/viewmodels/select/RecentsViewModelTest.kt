package com.gemwallet.android.features.assets.viewmodels.select

import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.RecentActivityQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.RecentActivityType
import com.wallet.core.primitives.RecentAsset
import com.wallet.core.primitives.WalletId
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
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
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemAssetFilter
import uniffi.gemstone.GemDay
import uniffi.gemstone.GemRecentActivityService
import uniffi.gemstone.GemRecentsDay
import uniffi.gemstone.GemRecentsSections
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
        every { viewState(any(), any(), any()) } answers {
            val recents = firstArg<List<uniffi.gemstone.RecentAsset>>()
            GemRecentsViewState(
                sections = GemRecentsSections(showsItems = recents.isNotEmpty(), showsClear = recents.isNotEmpty(), empty = null),
                days = secondArg<List<GemDay>>().firstOrNull()?.let { listOf(GemRecentsDay(it, recents)) }.orEmpty(),
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
    fun `the view state keeps content after dismiss`() = runTest(testDispatcher) {
        every { recentActivityQuery(WalletId("wallet-1"), any(), any(), 0) } returns flowOf(recentItems)
        val vm = RecentsViewModel(getCurrentWalletId, recentActivityQuery, recentActivityService, testDispatcher)

        vm.show()
        vm.viewState.first { it.days.isNotEmpty() }

        vm.dismiss()
        withContext(Dispatchers.Default) { delay(100) }
        assertEquals(recentItems.map { it.toGem() }, vm.viewState.value.days.flatMap { it.recents })
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
}

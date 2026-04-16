package com.gemwallet.android.features.asset_select.viewmodels

import com.gemwallet.android.application.asset_select.coordinators.ClearRecentAssets
import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.features.asset_select.viewmodels.models.RecentsEmptyState
import com.gemwallet.android.features.asset_select.viewmodels.models.RecentsSheetUIModel
import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.RecentAsset
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.testkit.mockAsset
import com.wallet.core.primitives.Chain
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.collections.immutable.persistentListOf
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class RecentsSheetViewModelTest {

    private val testDispatcher = UnconfinedTestDispatcher()

    private val solAsset = mockAsset(chain = Chain.Solana, name = "Solana", symbol = "SOL")
    private val ethAsset = mockAsset(chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH")
    private val recentItems = listOf(
        RecentAsset(asset = solAsset, addedAt = 1000L),
        RecentAsset(asset = ethAsset, addedAt = 2000L),
    )

    private val getRecentAssets = mockk<GetRecentAssets>(relaxed = true)
    private val clearRecentAssets = mockk<ClearRecentAssets>(relaxed = true)

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
        val vm = RecentsSheetViewModel(getRecentAssets, clearRecentAssets)

        assertFalse(vm.visible.value)

        vm.show(filters = setOf(AssetFilter.HasBalance))
        advanceUntilIdle()
        assertTrue(vm.visible.value)

        vm.dismiss()
        advanceUntilIdle()
        assertFalse(vm.visible.value)
    }

    @Test
    fun `clear delegates to coordinator with current types`() = runTest(testDispatcher) {
        val vm = RecentsSheetViewModel(getRecentAssets, clearRecentAssets)
        val types = listOf(RecentType.Swap)
        vm.show(types = types)
        advanceUntilIdle()

        vm.onClear()
        advanceUntilIdle()

        coVerify { clearRecentAssets(types) }
    }

    @Test
    fun `uiModel properties derive correctly`() {
        val withItems = RecentsSheetUIModel(
            items = recentItems.toImmutableList(),
            hasAnyRecents = true,
            searchActive = false,
        )
        assertFalse(withItems.isEmpty)
        assertTrue(withItems.showClear)
        assertNull(withItems.emptyState)

        val searchNoResults = RecentsSheetUIModel(
            items = persistentListOf(),
            hasAnyRecents = true,
            searchActive = true,
        )
        assertTrue(searchNoResults.isEmpty)
        assertFalse(searchNoResults.showClear)
        assertEquals(RecentsEmptyState.NoSearchResults, searchNoResults.emptyState)

        val noRecents = RecentsSheetUIModel(
            items = persistentListOf(),
            hasAnyRecents = false,
            searchActive = false,
        )
        assertTrue(noRecents.isEmpty)
        assertFalse(noRecents.showClear)
        assertEquals(RecentsEmptyState.NoRecents, noRecents.emptyState)
    }
}

package com.gemwallet.android.features.swap.viewmodels

import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.snapshots.Snapshot
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.asset_select.coordinators.GetRecentAssets
import com.gemwallet.android.application.asset_select.coordinators.SwitchAssetVisibility
import com.gemwallet.android.application.asset_select.coordinators.ToggleAssetPin
import com.gemwallet.android.application.asset_select.coordinators.UpdateRecentAsset
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.application.swap.coordinators.SearchSwapAssets
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.ui.models.navigation.RouteArgument
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class SwapSelectViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    private val getSession = mockk<GetSession>()
    private val getRecentAssets = mockk<GetRecentAssets>()
    private val updateRecentAsset = mockk<UpdateRecentAsset>(relaxed = true)
    private val switchAssetVisibility = mockk<SwitchAssetVisibility>(relaxed = true)
    private val toggleAssetPin = mockk<ToggleAssetPin>(relaxed = true)
    private val searchTokensCase = mockk<SearchTokensCase>()
    private val searchSwapAssets = mockk<SearchSwapAssets>()

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        every { getSession() } returns MutableStateFlow(null)
        every { getRecentAssets(any()) } returns flowOf(emptyList())
        every { searchSwapAssets(any(), any(), any(), any(), any()) } returns flowOf(emptyList())
        coEvery { searchTokensCase.search(any<String>(), any(), any(), any()) } returns true
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `pay selector does not start token network search`() = runTest(testDispatcher) {
        val viewModel = createViewModel(SwapItemType.Pay)
        advanceUntilIdle()

        viewModel.queryState.setTextAndPlaceCursorAtEnd("eth")
        Snapshot.sendApplyNotifications()
        advanceUntilIdle()

        coVerify(exactly = 0) { searchTokensCase.search(any<String>(), any(), any(), any()) }
    }

    private fun createViewModel(type: SwapItemType) = SwapSelectViewModel(
        getSession = getSession,
        getRecentAssets = getRecentAssets,
        updateRecentAsset = updateRecentAsset,
        switchAssetVisibility = switchAssetVisibility,
        toggleAssetPin = toggleAssetPin,
        searchTokensCase = searchTokensCase,
        searchSwapAssets = searchSwapAssets,
        savedStateHandle = SavedStateHandle(
            mapOf(RouteArgument.SwapItemType.key to type)
        ),
    )
}

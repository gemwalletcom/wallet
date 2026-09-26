package com.gemwallet.android.features.swap.viewmodels

import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.snapshots.Snapshot
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetsQuery
import com.gemwallet.android.data.services.store.queries.RecentActivityQuery
import com.gemwallet.android.domains.swap.SwapItemType
import com.gemwallet.android.testkit.mockGemEmptyState
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
import uniffi.gemstone.GemAssetSelectionServiceInterface
import uniffi.gemstone.GemSelectAssetType
import uniffi.gemstone.GemSelectAssetWalletFlow

@OptIn(ExperimentalCoroutinesApi::class)
class SwapSelectViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    private val getSession = mockk<GetSession>()
    private val recentActivityQuery = mockk<RecentActivityQuery>()
    private val service = mockk<GemAssetSelectionServiceInterface>()
    private val assetsQuery = mockk<AssetsQuery>()

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
        every { getSession() } returns MutableStateFlow(null)
        every { recentActivityQuery(any(), any(), any(), any()) } returns flowOf(emptyList())
        every { assetsQuery(any(), any(), any(), any(), any()) } returns flowOf(emptyList())
        every { service.flow(any()) } answers { firstArg<GemSelectAssetType>().flow() }
        coEvery { service.searchAssets(any()) } returns emptyList()
        every { service.walletFlow(any(), any()) } answers { GemSelectAssetWalletFlow(flow = firstArg<GemSelectAssetType>().flow(), chains = emptyList(), showsAddToken = false, showsChainFilter = false, emptyState = mockGemEmptyState()) }
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

        coVerify(exactly = 0) { service.searchAssets(any()) }
    }

    private fun createViewModel(type: SwapItemType) = SwapSelectViewModel(
        getSession = getSession,
        recentActivityQuery = recentActivityQuery,
        service = service,
        assetsQuery = assetsQuery,
        ioDispatcher = testDispatcher,
        context = mockk(relaxed = true),
        savedStateHandle = SavedStateHandle(
            mapOf(RouteArgument.SwapItemType.key to type),
        ),
    )
}

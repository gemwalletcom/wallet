package com.gemwallet.android.features.swap.viewmodels

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.swap.SwapRepository
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.swap.viewmodels.cases.QuoteRequester
import com.gemwallet.android.features.swap.viewmodels.models.SwapItemType
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class SwapViewModelTest {

    private val testDispatcher = StandardTestDispatcher()

    private val solAsset = mockAssetSolana()
    private val usdcAsset = mockAssetSolanaUSDC()
    private val solInfo = mockAssetInfo(asset = solAsset)
    private val usdcInfo = mockAssetInfo(asset = usdcAsset)

    private val sessionRepository = mockk<SessionRepository>(relaxed = true) {
        every { session() } returns MutableStateFlow(null)
    }
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true) {
        every { getAssetInfo(solAsset.id) } returns flowOf(solInfo)
        every { getAssetInfo(usdcAsset.id) } returns flowOf(usdcInfo)
    }
    private val swapRepository = mockk<SwapRepository>(relaxed = true)
    private val quoteRequester = mockk<QuoteRequester>(relaxed = true) {
        every { requestQuotes(any(), any(), any(), any(), any()) } returns emptyFlow()
    }

    @Before
    fun setUp() {
        Dispatchers.setMain(testDispatcher)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    private fun createViewModel(savedStateHandle: SavedStateHandle) = SwapViewModel(
        sessionRepository = sessionRepository,
        assetsRepository = assetsRepository,
        swapRepository = swapRepository,
        quoteRequester = quoteRequester,
        savedStateHandle = savedStateHandle,
    )

    @Test
    fun `onSelect updates pay asset from empty state`() = runTest(testDispatcher) {
        val savedState = SavedStateHandle()

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        viewModel.onSelect(SwapItemType.Pay, solAsset.id)
        advanceUntilIdle()

        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>("from"))
        assertNull(savedState.get<String?>("to"))
        assertEquals(solAsset.id, viewModel.payAsset.value?.id())
    }

    @Test
    fun `onSelect keeps opposite asset when pair differs`() = runTest(testDispatcher) {
        val savedState = SavedStateHandle(
            mapOf("from" to solAsset.id.toIdentifier(), "to" to usdcAsset.id.toIdentifier())
        )

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        viewModel.onSelect(SwapItemType.Receive, usdcAsset.id)
        advanceUntilIdle()

        assertEquals(usdcAsset.id.toIdentifier(), savedState.get<String?>("to"))
        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>("from"))
    }

    @Test
    fun `selecting same asset for both pay and receive clears opposite`() = runTest(testDispatcher) {
        val savedState = SavedStateHandle(
            mapOf("from" to solAsset.id.toIdentifier(), "to" to usdcAsset.id.toIdentifier())
        )

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        viewModel.onSelect(SwapItemType.Receive, solAsset.id)
        advanceUntilIdle()

        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>("to"))
        assertNull("pay must be cleared when receive matches it", savedState.get<String?>("from"))
    }

    @Test
    fun `selecting same pay asset clears receive`() = runTest(testDispatcher) {
        val savedState = SavedStateHandle(
            mapOf("from" to usdcAsset.id.toIdentifier(), "to" to solAsset.id.toIdentifier())
        )

        val viewModel = createViewModel(savedState)
        advanceUntilIdle()

        viewModel.onSelect(SwapItemType.Pay, solAsset.id)
        advanceUntilIdle()

        assertEquals(solAsset.id.toIdentifier(), savedState.get<String?>("from"))
        assertNull(savedState.get<String?>("to"))
    }
}

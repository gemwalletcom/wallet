package com.gemwallet.android.features.nft.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.nft.cases.GetNftCollections
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ToastMessage
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.launch
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
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemNftListScreen
import uniffi.gemstone.GemNftServiceInterface
import uniffi.gemstone.GemNftUnverifiedRow
import uniffi.gemstone.GemRefreshResult
import uniffi.gemstone.GemServiceException

@OptIn(ExperimentalCoroutinesApi::class)
class NftListViewModelsTest {

    private val dispatcher = StandardTestDispatcher()

    @Before
    fun setUp() {
        Dispatchers.setMain(dispatcher)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun `a failed refresh over only unverified collections toasts and keeps the list`() = runTest(dispatcher) {
        val onlyUnverified = GemNftListScreen(
            title = GemLocalizedText.NftCollections,
            offersReceive = true,
            syncsOnAppear = true,
            items = emptyList(),
            unverifiedRow = GemNftUnverifiedRow(countText = "1"),
            hasContent = true,
        )
        val service = mockk<GemNftServiceInterface> {
            every { listScreen(any(), any()) } returns onlyUnverified
            coEvery { refresh(true) } returns GemRefreshResult(state = GemLoadState.Data, toast = GemServiceException.Gateway("offline"))
        }
        val viewModel = NftListViewModels(
            nftService = service,
            getNftCollections = mockk<GetNftCollections> { every { this@mockk.invoke(any()) } returns flowOf(emptyList()) },
            getSession = mockk<GetSession> { every { this@mockk.invoke() } returns MutableStateFlow(null) },
            savedStateHandle = SavedStateHandle(),
            context = mockk<Context>(relaxed = true),
            ioDispatcher = dispatcher,
        )
        val toast = CompletableDeferred<ToastMessage>()
        val collector = launch { toast.complete(viewModel.toastEvents.first()) }

        viewModel.refresh()
        advanceUntilIdle()

        assertEquals(R.drawable.ic_error, toast.await().image)
        assertNull(viewModel.errorRow.value)
        collector.cancel()
    }
}

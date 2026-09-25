package com.gemwallet.android.features.nft.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.NFTQuery
import com.gemwallet.android.model.Session
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ToastMessage
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
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

    private val onlyUnverified = GemNftListScreen(
        title = GemLocalizedText.NftCollections,
        offersReceive = true,
        syncsOnAppear = true,
        items = emptyList(),
        unverifiedRow = GemNftUnverifiedRow(countText = "1"),
        hasContent = true,
    )

    @Test
    fun `a failed refresh over only unverified collections toasts and keeps the list`() = runTest(dispatcher) {
        val service = mockk<GemNftServiceInterface> {
            every { listScreen(any(), any()) } returns onlyUnverified
            coEvery { refresh(true) } returns GemRefreshResult(state = GemLoadState.Data, toast = GemServiceException.Gateway("offline"))
        }
        val viewModel = NftListViewModels(
            nftService = service,
            nftQuery = mockk<NFTQuery>(),
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

    @Test
    fun `collections follow the session wallet and ignore updates to the same wallet`() = runTest(dispatcher) {
        val service = mockk<GemNftServiceInterface> { every { listScreen(any(), any()) } returns onlyUnverified }
        val query = mockk<NFTQuery> { every { this@mockk.invoke(any(), any()) } returns flowOf(emptyList()) }
        val sessions = MutableStateFlow<Session?>(mockSession(wallet = mockWallet(id = WalletId("wallet-a"))))
        NftListViewModels(
            nftService = service,
            nftQuery = query,
            getSession = mockk<GetSession> { every { this@mockk.invoke() } returns sessions },
            savedStateHandle = SavedStateHandle(),
            context = mockk<Context>(relaxed = true),
            ioDispatcher = dispatcher,
        )
        advanceUntilIdle()

        sessions.value = mockSession(wallet = mockWallet(id = WalletId("wallet-b")))
        advanceUntilIdle()
        sessions.value = mockSession(wallet = mockWallet(id = WalletId("wallet-b"), name = "Renamed"))
        advanceUntilIdle()

        verify(exactly = 1) { query("wallet-a", null) }
        verify(exactly = 1) { query("wallet-b", null) }
    }
}

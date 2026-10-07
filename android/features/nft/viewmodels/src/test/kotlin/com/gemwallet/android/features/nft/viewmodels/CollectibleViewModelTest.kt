package com.gemwallet.android.features.nft.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.nft.cases.GetNftAssetDetails
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.domains.nft.NFTAssetDetails
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockNftAssetData
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.StateViewType
import com.gemwallet.android.ui.models.ToastMessage
import com.gemwallet.android.ui.models.dataOrNull
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.ReportReason
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemCollectibleDetails
import uniffi.gemstone.GemCollectibleServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class CollectibleViewModelTest {

    private val dispatcher = StandardTestDispatcher()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    @Test
    fun `a failed report shows the error`() = runTest(dispatcher) {
        val service = mockk<GemCollectibleServiceInterface> {
            coEvery { report(any(), any()) } throws RuntimeException("rate limited")
        }
        val viewModel = viewModel(details = emptyFlow(), service = service)
        val toast = CompletableDeferred<ToastMessage>()
        val collector = launch { toast.complete(viewModel.toastEvents.first()) }

        viewModel.report(ReportReason.Spam)

        assertEquals(R.drawable.ic_error, toast.await().image)
        collector.cancel()
    }

    @Test
    fun `a collectible that cannot be loaded shows the error instead of nothing`() = runTest(dispatcher) {
        val viewModel = viewModel(details = flow { throw RuntimeException("offline") })

        advanceUntilIdle()

        assertEquals(StateViewType.Error("offline"), viewModel.state.value)
    }

    @Test
    fun `a loaded collectible carries the details Core builds for it`() = runTest(dispatcher) {
        val assetData = mockNftAssetData()
        val details = mockk<GemCollectibleDetails>()
        val service = mockk<GemCollectibleServiceInterface> {
            every { details(any(), any(), true, any()) } returns details
        }
        val viewModel = viewModel(details = flowOf(NFTAssetDetails(assetData = assetData, isOwned = true)), service = service)

        assertTrue(viewModel.state.value is StateViewType.Loading)
        advanceUntilIdle()

        assertEquals(assetData, viewModel.state.value.dataOrNull?.assetData)
        assertEquals(details, viewModel.state.value.dataOrNull?.details)
    }

    private fun viewModel(details: Flow<NFTAssetDetails>, service: GemCollectibleServiceInterface = mockk()) = CollectibleViewModel(
        getNftAssetDetails = mockk<GetNftAssetDetails> { every { this@mockk.invoke(any()) } returns details },
        getSession = mockk<GetSession> { every { this@mockk.invoke() } returns MutableStateFlow(mockSession()) },
        service = service,
        savedStateHandle = SavedStateHandle(mapOf(RouteArgument.NftAssetId.key to "ethereum_0xcontract::1")),
        context = mockk<Context>(relaxed = true),
        ioDispatcher = dispatcher,
    )
}

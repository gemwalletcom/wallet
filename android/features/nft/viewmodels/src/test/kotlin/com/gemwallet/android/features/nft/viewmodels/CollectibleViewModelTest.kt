package com.gemwallet.android.features.nft.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.nft.cases.GetNftAssetDetails
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.models.ToastMessage
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.ReportReason
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemCollectibleServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class CollectibleViewModelTest {

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
    fun `a failed report shows the error`() = runTest(dispatcher) {
        val service = mockk<GemCollectibleServiceInterface> {
            coEvery { report(any(), any()) } throws RuntimeException("rate limited")
        }
        val details = mockk<GetNftAssetDetails> {
            every { this@mockk.invoke(any()) } returns emptyFlow()
        }
        val viewModel = CollectibleViewModel(
            getNftAssetDetails = details,
            getSession = mockk { every { this@mockk.invoke() } returns MutableStateFlow(null) },
            service = service,
            savedStateHandle = SavedStateHandle(mapOf(RouteArgument.NftAssetId.key to "ethereum_0xcontract::1")),
            context = mockk<Context>(relaxed = true),
            ioDispatcher = dispatcher,
        )
        val toast = CompletableDeferred<ToastMessage>()
        val collector = launch { toast.complete(viewModel.toastEvents.first()) }

        viewModel.report(ReportReason.Spam)

        assertEquals(R.drawable.ic_error, toast.await().image)
        collector.cancel()
    }
}

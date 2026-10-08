package com.gemwallet.android.features.transfer.viewmodels.confirm

import androidx.lifecycle.SavedStateHandle
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.serializer.packRoutePayload
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockPaymentLink
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.ui.models.navigation.RouteArgument
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemPaymentServiceInterface
import uniffi.gemstone.InternalException

@OptIn(ExperimentalCoroutinesApi::class)
class PaymentVerificationViewModelTest {

    private val dispatcher = UnconfinedTestDispatcher()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    @Test
    fun `an unexpected failure after verification shows a toast`() = runTest(dispatcher) {
        val paymentService = mockk<GemPaymentServiceInterface>()
        coEvery { paymentService.prepare(any(), any()) } throws InternalException("panic")
        val viewModel = PaymentVerificationViewModel(
            getSession = mockk<GetSession> { every { this@mockk() } returns MutableStateFlow(mockSession()) },
            paymentService = paymentService,
            savedStateHandle = SavedStateHandle(
                mapOf(
                    RouteArgument.PaymentLink.key to requireNotNull(mockPaymentLink().packRoutePayload()),
                    RouteArgument.Url.key to "https://verify.example",
                ),
            ),
            ioDispatcher = dispatcher,
            context = mockk(relaxed = true),
        )

        viewModel.verificationBridge.onDataCollectionComplete("""{"type":"IC_COMPLETE"}""")

        assertEquals("panic", viewModel.toastEvents.first().title)
        assertNull(viewModel.confirm.value)
    }
}

package com.gemwallet.android.features.wallet_connector.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequests
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockGemSignMessagePreview
import com.gemwallet.android.testkit.mockGemWalletConnectMessageRequest
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletConnectionSession
import com.gemwallet.android.ui.models.ButtonState
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemSignMessageServiceInterface
import uniffi.gemstone.GemWalletConnectServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
class SignMessageViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<SignMessageViewModel>()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
        Dispatchers.resetMain()
    }

    private val topic = "topic-1"

    private fun signMessageService(hasCriticalWarning: Boolean = false): GemSignMessageServiceInterface = mockk(relaxed = true) {
        every { preview(any()) } returns mockGemSignMessagePreview(hasCriticalWarning = hasCriticalWarning)
        coEvery { withAddressNames(any(), any()) } answers { secondArg() }
    }

    private fun viewModel(request: WalletConnectPendingRequest.SignMessage, service: GemWalletConnectServiceInterface = mockk(relaxed = true), signMessageService: GemSignMessageServiceInterface = signMessageService()) =
        SignMessageViewModel(
            request = request,
            service = service,
            signMessageService = signMessageService,
            ioDispatcher = dispatcher,
            context = mockk(relaxed = true),
        ).also { models.add(it) }

    private fun TestScope.pending(requests: WalletConnectPendingRequests, signature: CompletableDeferred<String>? = null): Job = launch {
        runCatching {
            requests.signMessage(mockGemWalletConnectMessageRequest(sessionId = topic, wallet = mockWallet(name = "Main Wallet").toGem(), session = mockWalletConnectionSession(sessionId = topic).toGem()))
        }.onSuccess { signature?.complete(it) }
    }

    private suspend fun WalletConnectPendingRequests.awaitMessage(): WalletConnectPendingRequest.SignMessage = current.filterNotNull().first() as WalletConnectPendingRequest.SignMessage

    @Test
    fun `a message without a critical warning can be signed`() = runTest(dispatcher) {
        val requests = WalletConnectPendingRequests()
        val job = pending(requests)
        val model = viewModel(requests.awaitMessage())
        advanceUntilIdle()

        assertEquals("Main Wallet", model.uiState.value.wallet.name)
        assertEquals(ButtonState.Enabled, model.buttonState.value)

        job.cancel()
    }

    @Test
    fun `a critical warning disables the sign button`() = runTest(dispatcher) {
        val requests = WalletConnectPendingRequests()
        val job = pending(requests)
        val model = viewModel(requests.awaitMessage(), signMessageService = signMessageService(hasCriticalWarning = true))

        assertEquals(ButtonState.Disabled, model.buttonState.first { it != ButtonState.Enabled })

        job.cancel()
    }

    @Test
    fun `a failed signature leaves the request open and only a cancelled one answers the dapp`() = runTest(dispatcher) {
        val requests = WalletConnectPendingRequests()
        val service = mockk<GemWalletConnectServiceInterface>(relaxed = true)
        coEvery { service.signMessage(any(), any()) } throws GemServiceException.Api("offline")
        val job = pending(requests)
        val model = viewModel(requests.awaitMessage(), service = service)

        val errors = mutableListOf<String>()
        val signatures = mutableListOf<String>()
        model.onSign(onSigned = signatures::add, onError = errors::add)
        advanceUntilIdle()

        assertEquals(1, errors.size)
        assertNotNull(requests.current.value)

        coEvery { service.signMessage(any(), any()) } throws GemServiceException.Cancelled()
        model.onSign(onSigned = signatures::add, onError = errors::add)
        job.join()

        assertEquals(1, errors.size)
        assertTrue(signatures.isEmpty())
        assertNull(requests.current.value)
    }

    @Test
    fun `signing hands the core signature on and keeps the button loading`() = runTest(dispatcher) {
        val requests = WalletConnectPendingRequests()
        val service = mockk<GemWalletConnectServiceInterface>(relaxed = true)
        coEvery { service.signMessage(any(), any()) } returns "0xdeadbeef"
        val job = pending(requests)
        val model = viewModel(requests.awaitMessage(), service = service)

        val signed = CompletableDeferred<String>()
        model.onSign(onSigned = { signed.complete(it) }, onError = {})

        assertEquals("0xdeadbeef", signed.await())
        assertEquals(ButtonState.Loading, model.buttonState.first { it == ButtonState.Loading })

        job.cancel()
    }

    @Test
    fun `rejecting is ignored while the message is signing`() = runTest(dispatcher) {
        val requests = WalletConnectPendingRequests()
        val service = mockk<GemWalletConnectServiceInterface>(relaxed = true)
        val signature = CompletableDeferred<String>()
        coEvery { service.signMessage(any(), any()) } coAnswers { signature.await() }
        val job = pending(requests)
        val model = viewModel(requests.awaitMessage(), service = service)

        val rejected = mutableListOf<Unit>()
        model.onReject { rejected.add(Unit) }
        model.onSign(onSigned = {}, onError = {})
        model.onReject { rejected.add(Unit) }
        signature.complete("0xdeadbeef")
        advanceUntilIdle()

        assertEquals(1, rejected.size)

        job.cancel()
    }
}

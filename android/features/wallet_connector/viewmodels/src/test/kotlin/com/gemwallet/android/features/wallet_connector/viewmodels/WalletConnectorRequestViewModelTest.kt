package com.gemwallet.android.features.wallet_connector.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequests
import com.gemwallet.android.application.wallet_connect.cases.RespondWalletConnectRequest
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.wallet_connector.viewmodels.models.WalletConnectorRequestUIState
import com.gemwallet.android.testkit.MainDispatcherRule
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockGemWalletConnectMessageRequest
import com.gemwallet.android.testkit.mockGemWalletConnectTransactionRequest
import com.gemwallet.android.testkit.mockSimulationResult
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletConnectSessionRequest
import com.gemwallet.android.testkit.mockWalletConnectVerifyContext
import com.gemwallet.android.testkit.mockWalletConnectionSession
import com.gemwallet.android.ui.R
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemWalletConnectFailure
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class WalletConnectorRequestViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<WalletConnectorRequestViewModel>()

    @get:Rule
    val mainDispatcherRule = MainDispatcherRule(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
    }

    private val topic = "topic-1"

    private val sessionRequest = mockWalletConnectSessionRequest(id = 42L, topic = topic)

    private val verifyContext = mockWalletConnectVerifyContext(origin = "https://app.uniswap.org")

    private fun respond(onRespond: ((GemWalletConnectFailure) -> Unit) -> Unit = {}): RespondWalletConnectRequest = mockk(relaxed = true) {
        every { respondRequest(any(), any(), any(), any()) } answers { onRespond(thirdArg()) }
    }

    private fun viewModel(respond: RespondWalletConnectRequest = respond(), requests: WalletConnectPendingRequests = WalletConnectPendingRequests()) = WalletConnectorRequestViewModel(
        respondWalletConnectRequest = respond,
        pendingRequests = requests,
        activeRequest = ActiveWalletConnectRequest(events = emptyFlow()),
        context = mockk(relaxed = true) {
            every { getString(R.string.errors_connections_malicious_origin) } returns "Malicious origin"
            every { getString(R.string.wallet_connect_request_expired) } returns "Request expired"
        },
    ).also { models.add(it) }

    private fun TestScope.pending(requests: WalletConnectPendingRequests, signature: CompletableDeferred<String>? = null): Job = launch {
        runCatching {
            requests.signMessage(mockGemWalletConnectMessageRequest(sessionId = topic, wallet = mockWallet(name = "Main Wallet").toGem(), session = mockWalletConnectionSession(sessionId = topic).toGem()))
        }.onSuccess { signature?.complete(it) }
    }

    private suspend fun WalletConnectorRequestViewModel.awaitContent(): WalletConnectorRequestUIState = uiState.first { it !is WalletConnectorRequestUIState.Loading }

    @Test
    fun `every failure Core reports is shown with its text`() = runTest(dispatcher) {
        listOf(
            GemWalletConnectFailure.MaliciousOrigin to "Malicious origin",
            GemWalletConnectFailure.Expired to "Request expired",
            GemWalletConnectFailure.Failed(GemErrorText.Message("Request failed")) to "Request failed",
        ).forEach { (failure, text) ->
            val errors = mutableListOf<String>()

            viewModel(respond = respond { onFailure -> onFailure(failure) }).onRequest(sessionRequest, verifyContext, onError = errors::add)

            assertEquals(listOf(text), errors)
        }
    }

    @Test
    fun `the same request is not started again`() = runTest(dispatcher) {
        val requests = WalletConnectPendingRequests()
        val respond = respond()
        val model = viewModel(respond = respond, requests = requests)

        model.onRequest(sessionRequest, verifyContext, onError = {})
        val job = pending(requests)
        model.awaitContent()

        model.onRequest(sessionRequest, verifyContext, onError = {})
        advanceUntilIdle()

        assertNotNull(requests.current.value)
        verify(exactly = 1) { respond.respondRequest(sessionRequest, verifyContext, any(), any()) }

        job.cancel()
    }

    @Test
    fun `a pending message request on the topic opens the sign message screen`() = runTest(dispatcher) {
        val requests = WalletConnectPendingRequests()
        val model = viewModel(requests = requests)

        model.onRequest(sessionRequest, verifyContext, onError = {})
        val job = pending(requests)

        val state = model.awaitContent()
        assertTrue(state is WalletConnectorRequestUIState.SignMessage)
        assertEquals("Main Wallet", (state as WalletConnectorRequestUIState.SignMessage).request.wallet.name)

        job.cancel()
    }

    @Test
    fun `a pending transaction request on the topic opens the confirm screen with its transfer`() = runTest(dispatcher) {
        val requests = WalletConnectPendingRequests()
        val model = viewModel(requests = requests)
        val wallet = mockWallet(name = "Main Wallet")
        val transfer = mockGemTransferData(value = BigInteger.TEN)
        val simulation = mockSimulationResult()

        model.onRequest(sessionRequest, verifyContext, onError = {})
        val job = launch {
            runCatching {
                requests.signTransaction(mockGemWalletConnectTransactionRequest(sessionId = topic, wallet = wallet.toGem(), session = mockWalletConnectionSession(sessionId = topic).toGem(), simulation = simulation, transfer = transfer))
            }
        }

        assertEquals(
            WalletConnectorRequestUIState.Transaction(ConfirmTransferInput(transfer, wallet), simulation),
            model.awaitContent(),
        )

        job.cancel()
    }

    @Test
    fun `rejecting rejects the pending request without responding`() = runTest(dispatcher) {
        val requests = WalletConnectPendingRequests()
        val respond = respond()
        val model = viewModel(respond = respond, requests = requests)

        model.onRequest(sessionRequest, verifyContext, onError = {})
        val job = pending(requests)
        model.awaitContent()

        model.onReject()
        job.join()

        assertNull(requests.current.value)
        verify(exactly = 0) { respond.rejectRequest(any(), any()) }
    }

    @Test
    fun `a result approves the pending request and keeps its screen while the dapp is answered`() = runTest(dispatcher) {
        val requests = WalletConnectPendingRequests()
        val model = viewModel(requests = requests)

        model.onRequest(sessionRequest, verifyContext, onError = {})
        val signature = CompletableDeferred<String>()
        val job = pending(requests, signature)
        model.awaitContent()

        model.onResult("0xdeadbeef")

        assertEquals("0xdeadbeef", signature.await())
        job.join()
        advanceUntilIdle()
        assertNull(requests.current.value)
        assertTrue(model.uiState.value is WalletConnectorRequestUIState.SignMessage)
    }

    @Test
    fun `rejecting a request still being answered refuses it, an answered one is left alone`() = runTest(dispatcher) {
        val answering = respond()
        val answered = mockk<RespondWalletConnectRequest>(relaxed = true) {
            every { respondRequest(any(), any(), any(), any()) } answers { arg<() -> Unit>(3)() }
        }

        viewModel(respond = answering).apply { onRequest(sessionRequest, verifyContext, onError = {}) }.onReject()
        viewModel(respond = answered).apply { onRequest(sessionRequest, verifyContext, onError = {}) }.onReject()

        verify(exactly = 1) { answering.rejectRequest(sessionRequest, any()) }
        verify(exactly = 0) { answered.rejectRequest(any(), any()) }
    }
}

package com.gemwallet.android.features.wallet_connector.viewmodels

import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectJsonRpcResponse
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequests
import com.gemwallet.android.application.wallet_connect.cases.RespondWalletConnectRequest
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.wallet_connector.viewmodels.models.WalletConnectorRequestUIState
import com.gemwallet.android.testkit.mockGemTransferData
import com.gemwallet.android.testkit.mockGemWalletConnectMessageRequest
import com.gemwallet.android.testkit.mockGemWalletConnectTransactionRequest
import com.gemwallet.android.testkit.mockSimulationResult
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletConnectSessionRequest
import com.gemwallet.android.testkit.mockWalletConnectVerifyContext
import com.gemwallet.android.testkit.mockWalletConnectionSession
import com.gemwallet.android.ui.R
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.emptyFlow
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
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemWalletConnectFailure
import uniffi.gemstone.GemWalletConnectOutcome
import uniffi.gemstone.GemWalletConnectResponse
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.GemWalletConnectSessionRequest
import uniffi.gemstone.WalletConnectResponseType
import uniffi.gemstone.WalletConnectionVerificationStatus
import java.math.BigInteger

@OptIn(ExperimentalCoroutinesApi::class)
class WalletConnectorRequestViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<WalletConnectorRequestViewModel>()

    @Before
    fun setUp() = Dispatchers.setMain(dispatcher)

    @After
    fun tearDown() {
        models.forEach { it.viewModelScope.cancel() }
        models.clear()
        Dispatchers.resetMain()
    }

    private val topic = "topic-1"

    private val idle = GemWalletConnectOutcome(response = null, failure = null)

    private val sessionRequest = mockWalletConnectSessionRequest(id = 42L, topic = topic)

    private val verifyContext = mockWalletConnectVerifyContext(origin = "https://app.uniswap.org")

    private fun service(onProcess: suspend (GemWalletConnectSessionRequest) -> GemWalletConnectOutcome = { idle }): GemWalletConnectServiceInterface = mockk(relaxed = true) {
        coEvery { requestOutcome(any()) } coAnswers { onProcess(firstArg()) }
    }

    private fun viewModel(service: GemWalletConnectServiceInterface = service(), respond: RespondWalletConnectRequest = mockk(relaxed = true), requests: WalletConnectPendingRequests = WalletConnectPendingRequests()) =
        WalletConnectorRequestViewModel(
            service = service,
            respondWalletConnectRequest = respond,
            pendingRequests = requests,
            activeRequest = ActiveWalletConnectRequest(events = emptyFlow()),
            ioDispatcher = dispatcher,
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
    fun `a malicious origin notifies without responding`() = runTest(dispatcher) {
        val notified = CompletableDeferred<String>()
        val respond = mockk<RespondWalletConnectRequest>(relaxed = true)
        val model = viewModel(
            service = service { GemWalletConnectOutcome(response = null, failure = GemWalletConnectFailure.MaliciousOrigin) },
            respond = respond,
        )

        model.onRequest(sessionRequest, verifyContext, onNotify = { notified.complete(it) }, onError = {})

        assertEquals("Malicious origin", notified.await())
        verify(exactly = 0) { respond.respond(any(), any(), any(), any(), any()) }
    }

    @Test
    fun `an expired request notifies as expired`() = runTest(dispatcher) {
        val notified = CompletableDeferred<String>()
        val model = viewModel(
            service = service { GemWalletConnectOutcome(response = null, failure = GemWalletConnectFailure.Expired) },
        )

        model.onRequest(sessionRequest, verifyContext, onNotify = { notified.complete(it) }, onError = {})

        assertEquals("Request expired", notified.await())
    }

    @Test
    fun `a failed request reports an error without a notification`() = runTest(dispatcher) {
        val error = CompletableDeferred<String>()
        val notified = mutableListOf<String>()
        val model = viewModel(
            service = service { GemWalletConnectOutcome(response = null, failure = GemWalletConnectFailure.Failed(GemErrorText.Message("Request failed"))) },
        )

        model.onRequest(sessionRequest, verifyContext, onNotify = { notified.add(it) }, onError = { error.complete(it) })

        assertEquals("Request failed", error.await())
        assertTrue(notified.isEmpty())
    }

    @Test
    fun `the request handed to core keeps the topic method params and origin`() = runTest(dispatcher) {
        val sent = CompletableDeferred<GemWalletConnectSessionRequest>()
        val model = viewModel(
            service = service { request ->
                sent.complete(request)
                GemWalletConnectOutcome(response = null, failure = null)
            },
        )

        model.onRequest(sessionRequest, verifyContext, onNotify = {}, onError = {})

        val request = sent.await()
        assertEquals(topic, request.topic)
        assertEquals("42", request.requestId)
        assertEquals("personal_sign", request.method)
        assertEquals("[]", request.params)
        assertEquals("eip155:1", request.chainId)
        assertEquals("https://app.uniswap.org", request.origin)
        assertEquals(WalletConnectionVerificationStatus.VERIFIED, request.validation)
    }

    @Test
    fun `a core response is sent back on the request topic`() = runTest(dispatcher) {
        val responded = CompletableDeferred<Triple<String, Long, WalletConnectJsonRpcResponse>>()
        val respond = mockk<RespondWalletConnectRequest>(relaxed = true) {
            every { respond(any(), any(), any(), any(), any()) } answers {
                responded.complete(Triple(firstArg(), secondArg(), thirdArg()))
                Unit
            }
        }
        val model = viewModel(
            service = service {
                GemWalletConnectOutcome(
                    response = GemWalletConnectResponse.Response(WalletConnectResponseType.String("0xsignature")),
                    failure = null,
                )
            },
            respond = respond,
        )

        model.onRequest(sessionRequest, verifyContext, onNotify = {}, onError = {})

        assertEquals(
            Triple(topic, 42L, WalletConnectJsonRpcResponse.Result("0xsignature")),
            responded.await(),
        )
    }

    @Test
    fun `the same request is not started again`() = runTest(dispatcher) {
        val requests = WalletConnectPendingRequests()
        val service = service()
        val model = viewModel(service = service, requests = requests)

        model.onRequest(sessionRequest, verifyContext, onNotify = {}, onError = {})
        val job = pending(requests)
        model.awaitContent()

        model.onRequest(sessionRequest, verifyContext, onNotify = {}, onError = {})
        advanceUntilIdle()

        assertNotNull(requests.current.value)
        coVerify(exactly = 1) { service.requestOutcome(any()) }

        job.cancel()
    }

    @Test
    fun `a pending message request on the topic opens the sign message screen`() = runTest(dispatcher) {
        val requests = WalletConnectPendingRequests()
        val model = viewModel(requests = requests)

        model.onRequest(sessionRequest, verifyContext, onNotify = {}, onError = {})
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

        model.onRequest(sessionRequest, verifyContext, onNotify = {}, onError = {})
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
        val respond = mockk<RespondWalletConnectRequest>(relaxed = true)
        val model = viewModel(respond = respond, requests = requests)

        model.onRequest(sessionRequest, verifyContext, onNotify = {}, onError = {})
        val job = pending(requests)
        model.awaitContent()

        model.onReject()
        job.join()

        assertNull(requests.current.value)
        verify(exactly = 0) { respond.respond(any(), any(), any(), any(), any()) }
    }

    @Test
    fun `a result approves the pending request and keeps its screen while the dapp is answered`() = runTest(dispatcher) {
        val requests = WalletConnectPendingRequests()
        val model = viewModel(requests = requests)

        model.onRequest(sessionRequest, verifyContext, onNotify = {}, onError = {})
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
}

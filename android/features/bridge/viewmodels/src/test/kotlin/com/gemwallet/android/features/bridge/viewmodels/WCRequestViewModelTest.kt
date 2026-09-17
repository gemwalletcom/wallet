package com.gemwallet.android.features.bridge.viewmodels

import uniffi.gemstone.GemApplicationMetadataServiceInterface
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_connect.ActiveWalletConnectRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectJsonRpcResponse
import com.gemwallet.android.application.wallet_connect.WalletConnectPendingRequests
import com.gemwallet.android.application.wallet_connect.cases.RespondWalletConnectRequest
import com.gemwallet.android.features.bridge.viewmodels.model.BridgeRequestError
import com.gemwallet.android.testkit.mockGemConnectionRow
import com.gemwallet.android.testkit.mockGemSignMessagePreview
import com.gemwallet.android.testkit.mockGemWalletConnectMessageRequest
import com.gemwallet.android.testkit.mockWalletConnectSessionRequest
import com.gemwallet.android.testkit.mockWalletConnectVerifyContext
import com.gemwallet.android.testkit.mockWalletConnectionSession
import com.gemwallet.android.ui.models.ButtonState
import io.mockk.coEvery
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
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.GemSignMessageServiceInterface
import uniffi.gemstone.GemWalletConnectFailure
import uniffi.gemstone.GemWalletConnectOutcome
import uniffi.gemstone.GemWalletConnectResponse
import uniffi.gemstone.GemWalletConnectRpcError
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.GemWalletConnectSessionRequest
import uniffi.gemstone.WalletConnectResponseType
import uniffi.gemstone.WalletConnectionVerificationStatus

@OptIn(ExperimentalCoroutinesApi::class)
class WCRequestViewModelTest {

    private val dispatcher = StandardTestDispatcher()
    private val models = mutableListOf<WCRequestViewModel>()

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

    private val verifyContext = mockWalletConnectVerifyContext()

    private fun metadataService(): GemApplicationMetadataServiceInterface = mockk {
        every { connectionRow(any()) } returns mockGemConnectionRow()
    }

    private fun service(
        onProcess: suspend (GemWalletConnectSessionRequest) -> GemWalletConnectOutcome = { idle },
    ): GemWalletConnectServiceInterface = mockk(relaxed = true) {
        every { userRejectedError() } returns GemWalletConnectRpcError(code = 4001, message = "User rejected")
        coEvery { processRequest(any()) } coAnswers { onProcess(firstArg()) }
    }

    private fun signMessageService(hasCriticalWarning: Boolean = false): GemSignMessageServiceInterface = mockk(relaxed = true) {
        every { preview(any(), any(), any()) } returns mockGemSignMessagePreview(hasCriticalWarning)
        coEvery { addressNames(any(), any()) } returns emptyList()
    }

    private fun viewModel(
        service: GemWalletConnectServiceInterface = service(),
        signMessageService: GemSignMessageServiceInterface = signMessageService(),
        respond: RespondWalletConnectRequest = mockk(relaxed = true),
        requests: WalletConnectPendingRequests = WalletConnectPendingRequests(),
    ) = WCRequestViewModel(
        service = service,
        metadataService = metadataService(),
        signMessageService = signMessageService,
        respondWalletConnectRequest = respond,
        pendingRequests = requests,
        activeRequest = ActiveWalletConnectRequest(events = emptyFlow()),
        ioDispatcher = dispatcher,
        context = mockk(relaxed = true),
    ).also { models.add(it) }

    private fun TestScope.pending(requests: WalletConnectPendingRequests, signature: CompletableDeferred<String>? = null): Job =
        launch { runCatching { requests.signMessage(mockGemWalletConnectMessageRequest(session = mockWalletConnectionSession(sessionId = topic))) }.onSuccess { signature?.complete(it) } }

    private suspend fun WCRequestViewModel.awaitContent(): RequestSceneState.Content =
        sceneState.first { it !is RequestSceneState.Loading } as RequestSceneState.Content

    @Test
    fun `a malicious origin notifies without responding`() = runTest(dispatcher) {
        val notified = CompletableDeferred<BridgeRequestError>()
        val respond = mockk<RespondWalletConnectRequest>(relaxed = true)
        val model = viewModel(
            service = service { GemWalletConnectOutcome(response = null, failure = GemWalletConnectFailure.MaliciousOrigin) },
            respond = respond,
        )

        model.onRequest(sessionRequest, verifyContext, onNotify = { notified.complete(it) }, onError = {})

        assertEquals(BridgeRequestError.MaliciousSession, notified.await())
        verify(exactly = 0) { respond.respond(any(), any(), any(), any(), any()) }
    }

    @Test
    fun `an expired request notifies as expired`() = runTest(dispatcher) {
        val notified = CompletableDeferred<BridgeRequestError>()
        val model = viewModel(
            service = service { GemWalletConnectOutcome(response = null, failure = GemWalletConnectFailure.Expired) },
        )

        model.onRequest(sessionRequest, verifyContext, onNotify = { notified.complete(it) }, onError = {})

        assertEquals(BridgeRequestError.Expired, notified.await())
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
    fun `a pending request on the topic becomes the scene content`() = runTest(dispatcher) {
        val requests = WalletConnectPendingRequests()
        val model = viewModel(requests = requests)

        model.onRequest(sessionRequest, verifyContext, onNotify = {}, onError = {})
        val job = pending(requests)

        val scene = model.awaitContent()
        assertTrue(scene is RequestSceneState.Request)
        assertEquals("Main Wallet", scene.walletName)
        assertEquals(ButtonState.Enabled, model.buttonState.value)

        job.cancel()
    }

    @Test
    fun `a critical warning disables the sign button`() = runTest(dispatcher) {
        val requests = WalletConnectPendingRequests()
        val model = viewModel(signMessageService = signMessageService(hasCriticalWarning = true), requests = requests)

        model.onRequest(sessionRequest, verifyContext, onNotify = {}, onError = {})
        val job = pending(requests)

        model.awaitContent()
        assertEquals(ButtonState.Disabled, model.buttonState.first { it != ButtonState.Enabled })

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
    fun `signing sends the core signature back to the pending request`() = runTest(dispatcher) {
        val requests = WalletConnectPendingRequests()
        val service = service()
        coEvery { service.signMessage(any(), any()) } returns "0xdeadbeef"
        val model = viewModel(service = service, requests = requests)

        model.onRequest(sessionRequest, verifyContext, onNotify = {}, onError = {})
        val signature = CompletableDeferred<String>()
        val job = pending(requests, signature)
        model.awaitContent()

        model.onSign(onError = {})

        assertEquals("0xdeadbeef", signature.await())
        job.join()
        val scene = model.sceneState.first { it is RequestSceneState.Responding }
        assertEquals("Main Wallet", (scene as RequestSceneState.Content).walletName)
        assertEquals(ButtonState.Loading, model.buttonState.first { it == ButtonState.Loading })
    }
}

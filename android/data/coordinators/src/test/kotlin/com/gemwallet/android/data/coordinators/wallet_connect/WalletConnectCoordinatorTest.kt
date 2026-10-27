package com.gemwallet.android.data.coordinators.wallet_connect

import com.gemwallet.android.application.wallet_connect.WalletConnectAuthenticationRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectClient
import com.gemwallet.android.application.wallet_connect.WalletConnectEvent
import com.gemwallet.android.application.wallet_connect.WalletConnectJsonRpcResponse
import com.gemwallet.android.application.wallet_connect.WalletConnectSession
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionNamespace
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionProposal
import com.gemwallet.android.application.wallet_connect.addingAccount
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.testkit.mockApplicationMetadata
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletConnectSessionProposal
import com.gemwallet.android.testkit.mockWalletConnectSessionRequest
import com.gemwallet.android.testkit.mockWalletConnectVerifyContext
import com.gemwallet.android.testkit.mockWalletConnectionSession
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletConnection
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.slot
import io.mockk.verify
import io.mockk.verifyOrder
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.withTimeout
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemChainService
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemSessionApproval
import uniffi.gemstone.GemWalletConnectFailure
import uniffi.gemstone.GemWalletConnectOutcome
import uniffi.gemstone.GemWalletConnectResponse
import uniffi.gemstone.GemWalletConnectRpcError
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.GemWalletConnectSessionRequest
import uniffi.gemstone.WalletConnectResponseType
import uniffi.gemstone.WalletConnectionVerificationStatus

@OptIn(ExperimentalCoroutinesApi::class)
class WalletConnectCoordinatorTest {

    private val wallet = mockWallet(id = WalletId("wallet-1"))
    private val metadata = mockApplicationMetadata()
    private val proposal = mockWalletConnectSessionProposal()
    private val sessionRequest = mockWalletConnectSessionRequest(id = 42L, topic = "topic-1")
    private val verifyContext = mockWalletConnectVerifyContext(origin = "https://app.uniswap.org")
    private val settledSession = WalletConnectSession(
        topic = "topic-1",
        expiry = 1_800_000_000,
        metadata = metadata,
        namespaces = mapOf("eip155" to WalletConnectSessionNamespace(chains = listOf("eip155:1"), methods = emptyList(), events = emptyList(), accounts = listOf("eip155:1:0xabc"))),
        redirect = null,
    )

    private val clientEvents = MutableSharedFlow<WalletConnectEvent>(extraBufferCapacity = 8)
    private val client = mockk<WalletConnectClient>(relaxed = true) {
        every { events } returns clientEvents
        every { initialize(any(), any()) } answers { firstArg<() -> Unit>()() }
        every { pair(any(), any(), any()) } answers { secondArg<() -> Unit>()() }
        every { activeSessions() } returns emptyList()
        every { generateApprovedNamespaces(proposal, any()) } returns settledSession.namespaces
        every { approveSession(any(), any(), any(), any(), any()) } answers { arg<() -> Unit>(3)() }
        every { approveAuthentication(any(), any(), any(), any()) } answers { arg<() -> Unit>(2)() }
        every { respondSessionRequest(any(), any(), any(), any(), any()) } answers { arg<() -> Unit>(3)() }
    }
    private val stored = linkedMapOf<String, WalletConnection>()
    private val walletConnectService = mockk<GemWalletConnectServiceInterface>(relaxed = true) {
        every { sessionApproval(any()) } returns GemSessionApproval(chains = emptyList(), accounts = emptyList(), methods = emptyList(), events = emptyList())
        coEvery { configSessionProperties(any(), any(), any()) } returns emptyMap()
        every { session(any(), any(), any(), any()) } answers {
            mockWalletConnectionSession(id = firstArg(), sessionId = firstArg(), expireAt = arg<Long>(2) * 1000).toGem()
        }
        coEvery { addConnection(any()) } answers { firstArg<uniffi.gemstone.WalletConnection>().toPrimitives().let { stored[it.session.id] = it } }
    }
    private val chainService = mockk<GemChainService>(relaxed = true)
    private val subject = WalletConnectCoordinator(
        walletConnectClient = client,
        walletConnectService = walletConnectService,
        chainService = chainService,
    )

    @Test
    fun `a settled session is stored once for the wallet that approved it`() = runBlocking {
        subject.pair("wc:uri")
        clientEvents.subscriptionCount.first { it > 0 }
        clientEvents.emit(WalletConnectEvent.SessionSettled(settledSession))

        approve(wallet, proposal)
        clientEvents.emit(WalletConnectEvent.SessionSettled(settledSession))
        clientEvents.emit(WalletConnectEvent.SessionSettled(settledSession))
        awaitHandled()

        assertEquals(listOf(wallet.id to "topic-1"), stored.values.map { it.wallet.id to it.session.sessionId })
    }

    @Test
    fun `sessions that settle out of order are stored for the wallet that approved each`() = runBlocking {
        val otherWallet = mockWallet(id = WalletId("wallet-2"))
        val otherProposal = proposal.copy(proposerPublicKey = "key-2")
        val otherSession = settledSession.copy(topic = "topic-2", namespaces = mapOf("eip155" to settledSession.namespaces.getValue("eip155").copy(accounts = listOf("eip155:1:0xdef"))))
        every { client.generateApprovedNamespaces(otherProposal, any()) } returns otherSession.namespaces
        subject.pair("wc:uri")
        clientEvents.subscriptionCount.first { it > 0 }

        approve(wallet, proposal)
        approve(otherWallet, otherProposal)
        clientEvents.emit(WalletConnectEvent.SessionSettled(otherSession))
        clientEvents.emit(WalletConnectEvent.SessionSettled(settledSession))
        awaitHandled()

        assertEquals(
            listOf(otherWallet.id to "topic-2", wallet.id to "topic-1"),
            stored.values.map { it.wallet.id to it.session.sessionId },
        )
    }

    @Test
    fun `two dapps approved with one wallet before either settles are both stored`() = runBlocking {
        val otherProposal = proposal.copy(proposerPublicKey = "key-2")
        every { client.generateApprovedNamespaces(otherProposal, any()) } returns settledSession.namespaces
        subject.pair("wc:uri")
        clientEvents.subscriptionCount.first { it > 0 }

        approve(wallet, proposal)
        approve(wallet, otherProposal)
        clientEvents.emit(WalletConnectEvent.SessionSettled(settledSession))
        clientEvents.emit(WalletConnectEvent.SessionSettled(settledSession.copy(topic = "topic-2")))
        awaitHandled()

        assertEquals(listOf(wallet.id to "topic-1", wallet.id to "topic-2"), stored.values.map { it.wallet.id to it.session.sessionId })
    }

    @Test
    fun `an approval Core cannot prepare reports its error and never approves`() = runTest {
        coEvery { walletConnectService.configSessionProperties(any(), any(), any()) } throws GemServiceException.Store("disk full")
        val errors = mutableListOf<GemErrorText>()

        coordinator().approveConnection(wallet, proposal, onSuccess = {}, onError = errors::add)
        advanceUntilIdle()

        assertEquals(1, errors.size)
        verify(exactly = 0) { client.approveSession(any(), any(), any(), any(), any()) }
    }

    @Test
    fun `an authenticated session is stored for the wallet that approved it`() = runBlocking {
        val existingSession = settledSession.copy(topic = "topic-0")
        every { client.activeSessions() } returnsMany listOf(listOf(existingSession), listOf(existingSession, settledSession))
        val approved = CompletableDeferred<Unit>()

        subject.approveAuthentication(mockk<WalletConnectAuthenticationRequest>(), emptyList(), wallet, onSuccess = { approved.complete(Unit) }, onError = {})
        withTimeout(2_000) { approved.await() }

        assertEquals(listOf(wallet.id to "topic-1"), stored.values.map { it.wallet.id to it.session.sessionId })
    }

    @Test
    fun `a proposal Core has already seen never reaches the screen`() = runBlocking {
        every { walletConnectService.shouldProcessProposal(proposal.proposerPublicKey) } returnsMany listOf(true, false)
        val verifyContext = mockWalletConnectVerifyContext()
        val request = mockWalletConnectSessionRequest()
        subject.pair("wc:uri")
        val delivered = async { subject.bridgeEvents.take(2).toList() }
        clientEvents.subscriptionCount.first { it > 1 }

        clientEvents.emit(WalletConnectEvent.SessionProposal(proposal, verifyContext))
        clientEvents.emit(WalletConnectEvent.SessionProposal(proposal.copy(), verifyContext))
        clientEvents.emit(WalletConnectEvent.SessionRequest(request, verifyContext))

        assertEquals(
            listOf(
                WalletConnectEvent.SessionProposal(proposal, verifyContext),
                WalletConnectEvent.SessionRequest(request, verifyContext),
            ),
            withTimeout(2_000) { delivered.await() },
        )
    }

    @Test
    fun `an extended or updated session and the connections screen re-sync through Core`() = runBlocking {
        subject.pair("wc:uri")
        clientEvents.subscriptionCount.first { it > 0 }

        clientEvents.emit(WalletConnectEvent.SessionChanged("topic-1"))
        coVerify(timeout = 2_000, exactly = 1) { walletConnectService.updateSessions(any()) }

        subject.syncSessions()
        coVerify(exactly = 2) { walletConnectService.updateSessions(any()) }
    }

    @Test
    fun `a failed session event leaves the following events handled`() = runBlocking {
        coEvery { walletConnectService.deleteSession("topic-1") } throws GemServiceException.Store("disk full")
        subject.pair("wc:uri")
        clientEvents.subscriptionCount.first { it > 0 }

        clientEvents.emit(WalletConnectEvent.SessionDeleted("topic-1"))
        clientEvents.emit(WalletConnectEvent.SessionChanged("topic-2"))

        coVerify(timeout = 2_000, exactly = 1) { walletConnectService.updateSessions(any()) }
    }

    @Test
    fun `a request goes to Core with its fields and Core's response goes back on its topic`() = runTest {
        val sent = slot<GemWalletConnectSessionRequest>()
        coEvery { walletConnectService.requestOutcome(capture(sent)) } returns GemWalletConnectOutcome.Respond(GemWalletConnectResponse.Response(WalletConnectResponseType.String("0xsignature")))

        coordinator().respondRequest(sessionRequest, verifyContext, onFailure = {}, onFinish = {})
        advanceUntilIdle()

        assertEquals(
            GemWalletConnectSessionRequest(
                topic = "topic-1",
                requestId = "42",
                method = "personal_sign",
                params = "[]",
                chainId = "eip155:1",
                origin = "https://app.uniswap.org",
                validation = WalletConnectionVerificationStatus.VERIFIED,
                expiry = null,
            ),
            sent.captured,
        )
        verify { client.respondSessionRequest("topic-1", 42L, WalletConnectJsonRpcResponse.Result("0xsignature"), any(), any()) }
    }

    @Test
    fun `a failed request is reported, answered with its error and finished even if the answer fails`() = runTest {
        val failures = mutableListOf<GemWalletConnectFailure>()
        var finished = false
        coEvery { walletConnectService.requestOutcome(any()) } returns GemWalletConnectOutcome.Fail(GemWalletConnectRpcError(code = 4001, message = "User rejected"), GemWalletConnectFailure.MaliciousOrigin)
        every { client.respondSessionRequest(any(), any(), any(), any(), any()) } answers { arg<(String) -> Unit>(4)("relay down") }

        coordinator().respondRequest(sessionRequest, verifyContext, onFailure = failures::add) { finished = true }
        advanceUntilIdle()

        assertEquals(listOf(GemWalletConnectFailure.MaliciousOrigin), failures)
        assertEquals(true, finished)
        verify { client.respondSessionRequest("topic-1", 42L, WalletConnectJsonRpcResponse.Error(4001, "User rejected"), any(), any()) }
    }

    @Test
    fun `a switch to a network outside the session adds the account before answering`() = runTest {
        every { client.activeSessions() } returns listOf(settledSession)
        every { client.updateSession(any(), any(), any(), any()) } answers { thirdArg<() -> Unit>()() }
        coEvery { walletConnectService.requestOutcome(any()) } returns GemWalletConnectOutcome.AddSessionAccount("eip155:81457:0xabc")

        coordinator().respondRequest(sessionRequest, verifyContext, onFailure = {}, onFinish = {})
        advanceUntilIdle()

        verifyOrder {
            client.updateSession("topic-1", settledSession.namespaces.addingAccount("eip155:81457:0xabc"), any(), any())
            client.respondSessionRequest("topic-1", 42L, WalletConnectJsonRpcResponse.Result("null"), any(), any())
        }
    }

    @Test
    fun `rejecting a request Core is still answering answers it once as rejected by the user`() = runTest {
        coEvery { walletConnectService.requestOutcome(any()) } coAnswers { awaitCancellation() }
        val coordinator = coordinator()

        coordinator.respondRequest(sessionRequest, verifyContext, onFailure = {}, onFinish = {})
        advanceUntilIdle()
        coordinator.rejectRequest(sessionRequest) {}
        coordinator.rejectRequest(sessionRequest) {}
        advanceUntilIdle()

        verify(exactly = 1) {
            client.respondSessionRequest("topic-1", 42L, WalletConnectJsonRpcResponse.Error(GemConstants.walletConnectUserRejectedErrorCode, GemConstants.walletConnectUserRejectedErrorMessage), any(), any())
        }
    }

    private fun TestScope.coordinator() = WalletConnectCoordinator(
        walletConnectClient = client,
        walletConnectService = walletConnectService,
        chainService = chainService,
        scope = TestScope(testScheduler),
    )

    private suspend fun awaitHandled() {
        clientEvents.emit(WalletConnectEvent.SessionChanged("sentinel"))
        coVerify(timeout = 2_000) { walletConnectService.updateSessions(any()) }
    }

    private suspend fun approve(wallet: Wallet, proposal: WalletConnectSessionProposal) {
        val approved = CompletableDeferred<Unit>()
        subject.approveConnection(wallet, proposal, onSuccess = { approved.complete(Unit) }, onError = { approved.completeExceptionally(AssertionError(it.toString())) })
        withTimeout(2_000) { approved.await() }
    }
}

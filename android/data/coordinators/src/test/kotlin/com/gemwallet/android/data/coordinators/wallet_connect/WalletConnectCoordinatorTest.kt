package com.gemwallet.android.data.coordinators.wallet_connect

import com.gemwallet.android.application.wallet_connect.WalletConnectAuthenticationRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectClient
import com.gemwallet.android.application.wallet_connect.WalletConnectEvent
import com.gemwallet.android.application.wallet_connect.WalletConnectSession
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionNamespace
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.testkit.mockApplicationMetadata
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletConnectSessionProposal
import com.gemwallet.android.testkit.mockWalletConnectSessionRequest
import com.gemwallet.android.testkit.mockWalletConnectVerifyContext
import com.gemwallet.android.testkit.mockWalletConnectionSession
import com.wallet.core.primitives.WalletConnection
import com.wallet.core.primitives.WalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemChainService
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemSessionApproval
import uniffi.gemstone.GemWalletConnectServiceInterface

class WalletConnectCoordinatorTest {

    private val wallet = mockWallet(id = WalletId("wallet-1"))
    private val metadata = mockApplicationMetadata()
    private val proposal = mockWalletConnectSessionProposal()
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
    private val subject = WalletConnectCoordinator(
        walletConnectClient = client,
        walletConnectService = walletConnectService,
        chainService = mockk<GemChainService>(relaxed = true),
    )

    @Test
    fun `a settled session is stored once for the wallet that approved it`() = runBlocking {
        subject.pair("wc:uri")
        clientEvents.subscriptionCount.first { it > 0 }
        clientEvents.emit(WalletConnectEvent.SessionSettled(settledSession))

        subject.approveConnection(wallet, proposal, onSuccess = {}, onError = {})
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

        subject.approveConnection(wallet, proposal, onSuccess = {}, onError = {})
        subject.approveConnection(otherWallet, otherProposal, onSuccess = {}, onError = {})
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

        subject.approveConnection(wallet, proposal, onSuccess = {}, onError = {})
        subject.approveConnection(wallet, otherProposal, onSuccess = {}, onError = {})
        clientEvents.emit(WalletConnectEvent.SessionSettled(settledSession))
        clientEvents.emit(WalletConnectEvent.SessionSettled(settledSession.copy(topic = "topic-2")))
        awaitHandled()

        assertEquals(listOf(wallet.id to "topic-1", wallet.id to "topic-2"), stored.values.map { it.wallet.id to it.session.sessionId })
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

    private suspend fun awaitHandled() {
        clientEvents.emit(WalletConnectEvent.SessionChanged("sentinel"))
        coVerify(timeout = 2_000) { walletConnectService.updateSessions(any()) }
    }
}

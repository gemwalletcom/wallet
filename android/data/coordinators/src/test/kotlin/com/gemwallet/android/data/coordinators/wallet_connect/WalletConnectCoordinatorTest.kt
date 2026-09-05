package com.gemwallet.android.data.coordinators.wallet_connect

import com.gemwallet.android.application.wallet_connect.WalletConnectClient
import com.gemwallet.android.application.wallet_connect.WalletConnectEvent
import com.gemwallet.android.application.wallet_connect.WalletConnectSession
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionNamespace
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionProposal
import com.gemwallet.android.data.services.gemstone.stores.GemstoneConnectionStore
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.ApplicationMetadata
import com.wallet.core.primitives.ApplicationMetadataSource
import com.wallet.core.primitives.WalletConnection
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemChainService
import uniffi.gemstone.GemSessionApproval
import uniffi.gemstone.GemWalletConnectServiceInterface
import uniffi.gemstone.WalletConnectionSession
import uniffi.gemstone.WalletConnectionState

class WalletConnectCoordinatorTest {

    private val wallet = mockWallet(id = "wallet-1")
    private val metadata = ApplicationMetadata(
        name = "Uniswap",
        description = "Swap",
        url = "https://app.uniswap.org",
        icon = "https://app.uniswap.org/icon.png",
        source = ApplicationMetadataSource.WalletConnect,
    )
    private val settledSession = WalletConnectSession(
        topic = "topic-1",
        expiry = 1_800_000_000,
        metadata = metadata,
        namespaces = mapOf("eip155" to WalletConnectSessionNamespace(chains = listOf("eip155:1"), methods = emptyList(), events = emptyList(), accounts = listOf("eip155:1:0xabc"))),
        redirect = null,
    )
    private val proposal = WalletConnectSessionProposal(
        name = metadata.name,
        description = metadata.description,
        url = metadata.url,
        icons = listOf(metadata.icon),
        requiredNamespaces = emptyMap(),
        optionalNamespaces = emptyMap(),
        proposerPublicKey = "proposer",
        properties = null,
    )

    private val clientEvents = MutableSharedFlow<WalletConnectEvent>(extraBufferCapacity = 8)
    private val client = mockk<WalletConnectClient>(relaxed = true) {
        every { events } returns clientEvents
        every { initialize(any(), any()) } answers { firstArg<() -> Unit>()() }
        every { pair(any(), any(), any()) } answers { secondArg<() -> Unit>()() }
        every { activeSessions() } returns emptyList()
        every { generateApprovedNamespaces(any(), any()) } returns emptyMap()
        every { approveSession(any(), any(), any(), any(), any()) } answers { arg<() -> Unit>(3)() }
    }
    private val stored = mutableListOf<WalletConnection>()
    private val connectionStore = mockk<GemstoneConnectionStore> {
        coEvery { getConnectionBySessionId(any()) } answers { stored.firstOrNull { it.session.sessionId == firstArg<String>() } }
    }
    private val walletConnectService = mockk<GemWalletConnectServiceInterface>(relaxed = true) {
        every { sessionApproval(any()) } returns GemSessionApproval(chains = emptyList(), accounts = emptyList(), methods = emptyList(), events = emptyList())
        every { configSessionProperties(any(), any(), any()) } returns emptyMap()
        every { session(any(), any(), any(), any()) } answers {
            WalletConnectionSession(
                id = firstArg(),
                sessionId = firstArg(),
                state = WalletConnectionState.ACTIVE,
                chains = listOf("ethereum"),
                createdAt = 0L,
                expireAt = arg<Long>(2) * 1000,
                metadata = metadata.toGem(),
            )
        }
        coEvery { addConnection(any()) } answers { stored += firstArg<uniffi.gemstone.WalletConnection>().toPrimitives() }
    }
    private val subject = WalletConnectCoordinator(
        connectionStore = connectionStore,
        walletConnectClient = client,
        walletConnectService = walletConnectService,
        chainService = mockk<GemChainService>(relaxed = true),
    )

    @Test
    fun `a settled session is stored once for the wallet that approved it`() = runBlocking {
        subject.pair("wc:uri")
        clientEvents.emit(WalletConnectEvent.SessionSettled(settledSession))

        subject.approveConnection(wallet, proposal, onSuccess = {}, onError = {})
        clientEvents.emit(WalletConnectEvent.SessionSettled(settledSession))
        coVerify(timeout = 2_000) { walletConnectService.addConnection(any()) }
        clientEvents.emit(WalletConnectEvent.SessionSettled(settledSession))
        Thread.sleep(200)

        assertEquals(1, stored.size)
        assertEquals(wallet.id, stored.single().wallet.id)
        assertEquals("topic-1", stored.single().session.sessionId)
    }
}

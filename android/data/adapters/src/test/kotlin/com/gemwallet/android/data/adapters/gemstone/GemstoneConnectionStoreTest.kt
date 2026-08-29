package com.gemwallet.android.data.adapters.gemstone

import com.gemwallet.android.data.service.store.database.ConnectionsDao
import com.gemwallet.android.data.service.store.database.entities.DbConnection
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletConnectionState
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.WalletConnectionSession

class GemstoneConnectionStoreTest {

    private val walletStore = mockk<GemstoneWalletStore>()
    private val connectionsDao = mockk<ConnectionsDao>(relaxed = true)
    private val store = GemstoneConnectionStore(walletStore, connectionsDao)

    @Test
    fun observeConnections_mapsOnlyRecordsWithMatchingWallets() = runTest {
        every { walletStore.observeWallets() } returns flowOf(listOf(mockWallet(id = "wallet-1")))
        every { connectionsDao.getAll() } returns flowOf(
            listOf(
                connection(id = "connection-1", walletId = "wallet-1"),
                connection(id = "connection-2", walletId = "missing-wallet"),
            )
        )

        val connections = store.observeConnections().first()

        assertEquals(listOf("connection-1"), connections.map { it.session.id })
        assertEquals("wallet-1", connections.single().wallet.id.id)
    }

    @Test
    fun getSessions_returnsEverySessionCoreStored() = runTest {
        coEvery { connectionsDao.getConnections() } returns listOf(
            connection(id = "connection-1", walletId = "wallet-1"),
            connection(id = "connection-2", walletId = "missing-wallet"),
        )

        val sessions = store.getSessions().map { it.decodeJson<WalletConnectionSession>() }

        assertEquals(listOf("connection-1", "connection-2"), sessions.map { it.id })
    }

    @Test
    fun getConnectionBySessionId_returnsNullForMissingWallet() = runTest {
        coEvery { connectionsDao.getBySessionId("topic-1") } returns connection(id = "topic-1", walletId = "missing-wallet")
        every { walletStore.observeWallets() } returns flowOf(listOf(mockWallet(id = "wallet-1")))

        assertNull(store.getConnectionBySessionId("topic-1"))
        assertNull(store.getConnection("topic-1"))
    }

    @Test
    fun updateSession_keepsWalletAndCreationDate() = runTest {
        val record = connection(id = "connection-1", walletId = "wallet-1")
        coEvery { connectionsDao.getBySessionId("connection-1") } returns record
        val session = record.toDTO(mockWallet(id = "wallet-1")).session.copy(chains = listOf(Chain.Ethereum, Chain.Solana), expireAt = 3_000)

        store.updateSession(session.toJson())

        coVerify { connectionsDao.insert(record.copy(chains = listOf(Chain.Ethereum, Chain.Solana), expireAt = 3_000)) }
    }

    @Test
    fun updateSession_ignoresUnknownSessions() = runTest {
        coEvery { connectionsDao.getBySessionId("missing") } returns null

        store.updateSession(connection(id = "missing", walletId = "wallet-1").toDTO(mockWallet(id = "wallet-1")).session.toJson())

        coVerify(exactly = 0) { connectionsDao.insert(any<DbConnection>()) }
    }

    private fun connection(
        id: String,
        walletId: String,
    ) = DbConnection(
        id = id,
        walletId = walletId,
        sessionId = id,
        state = WalletConnectionState.Active,
        chains = listOf(Chain.Ethereum),
        createdAt = 1_000,
        expireAt = 2_000,
        appName = "App",
        appDescription = "Description",
        appUrl = "https://example.com",
        appIcon = "https://example.com/icon.png",
        redirectNative = null,
        redirectUniversal = null,
    )
}

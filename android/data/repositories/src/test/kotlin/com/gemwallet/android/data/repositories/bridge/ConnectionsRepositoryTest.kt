package com.gemwallet.android.data.repositories.bridge

import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.data.service.store.database.ConnectionsDao
import com.gemwallet.android.data.service.store.database.entities.DbConnection
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

class ConnectionsRepositoryTest {

    private val walletsRepository = mockk<WalletsRepository>()
    private val connectionsDao = mockk<ConnectionsDao>(relaxed = true)
    private val repository = ConnectionsRepository(walletsRepository, connectionsDao)

    @Test
    fun getConnections_mapsOnlyRecordsWithMatchingWallets() = runTest {
        every { walletsRepository.getAll() } returns flowOf(listOf(mockWallet(id = "wallet-1")))
        every { connectionsDao.getAll() } returns flowOf(
            listOf(
                connection(id = "connection-1", walletId = "wallet-1"),
                connection(id = "connection-2", walletId = "missing-wallet"),
            )
        )

        val connections = repository.getConnections().first()

        assertEquals(listOf("connection-1"), connections.map { it.session.id })
        assertEquals("wallet-1", connections.single().wallet.id.id)
    }

    @Test
    fun getConnectionByTopic_returnsNullForMissingWallet() = runTest {
        coEvery { connectionsDao.getBySessionId("topic-1") } returns connection(id = "topic-1", walletId = "missing-wallet")
        every { walletsRepository.getAll() } returns flowOf(listOf(mockWallet(id = "wallet-1")))

        assertNull(repository.getConnectionByTopic("topic-1"))
    }

    @Test
    fun disconnect_deletesConnectionAndReturnsMappedConnection() = runTest {
        every { walletsRepository.getAll() } returns flowOf(listOf(mockWallet(id = "wallet-1")))
        every { connectionsDao.getAll() } returns flowOf(listOf(connection(id = "connection-1", walletId = "wallet-1")))

        val connection = repository.disconnect("connection-1")

        assertEquals("connection-1", connection?.session?.id)
        coVerify { connectionsDao.delete("connection-1") }
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

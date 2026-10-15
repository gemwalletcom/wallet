package com.gemwallet.android.data.services.store.integration

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.gemwallet.android.data.services.store.database.entities.mockDbConnection
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.data.services.store.database.mockGemDatabase
import com.gemwallet.android.data.services.store.queries.ConnectionQuery
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class ConnectionQueryTest {
    private val database = mockGemDatabase()
    private val query = ConnectionQuery(database.walletsDao(), database.connectionsDao())
    private val wallet = DbWallet(id = "wallet-1", name = "Wallet 1", domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import)
    private val connection = mockDbConnection(id = "connection-1", walletId = "wallet-1").copy(sessionId = "topic-1")

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database.walletsDao().insert(wallet)
        database.connectionsDao().insert(listOf(connection, mockDbConnection(id = "connection-2", walletId = "wallet-1")))
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun theConnectionIsFoundByItsIdWithItsWallet() = runBlocking(Dispatchers.IO) {
        assertEquals(connection.toDTO(wallet.toDTO(emptyList())), query("connection-1").first())
    }

    @Test
    fun aSessionTopicOrUnknownIdFindsNoConnection() = runBlocking(Dispatchers.IO) {
        assertNull(query("topic-1").first())
        assertNull(query("connection-3").first())
    }
}

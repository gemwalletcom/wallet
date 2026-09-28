package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.gemwallet.android.data.services.store.database.entities.mockDbConnection
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.data.services.store.queries.ConnectionsQuery
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class ConnectionsQueryTest {
    private val database = Room.inMemoryDatabaseBuilder(
        InstrumentationRegistry.getInstrumentation().targetContext,
        GemDatabase::class.java,
    ).build()
    private val query = ConnectionsQuery(database.walletsDao(), database.connectionsDao())
    private val wallet1 = DbWallet(id = "wallet-1", name = "Wallet 1", domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import)
    private val wallet2 = DbWallet(id = "wallet-2", name = "Wallet 2", domainName = null, type = WalletType.Multicoin, position = 1, pinned = false, index = 1, source = WalletSource.Import)

    @After
    fun tearDown() = database.close()

    @Test
    fun everyStoredConnectionComesWithItsWalletAcrossWallets() = runBlocking(Dispatchers.IO) {
        database.walletsDao().insert(wallet1)
        database.walletsDao().insert(wallet2)
        val first = mockDbConnection(id = "connection-b", walletId = "wallet-1")
        val second = mockDbConnection(id = "connection-a", walletId = "wallet-2")
        val third = mockDbConnection(id = "connection-c", walletId = "wallet-1").copy(sessionId = "topic-c")
        database.connectionsDao().insert(listOf(first, second, third))

        assertEquals(
            listOf(first.toDTO(wallet1.toDTO(emptyList())), second.toDTO(wallet2.toDTO(emptyList())), third.toDTO(wallet1.toDTO(emptyList()))),
            query().first(),
        )
    }
}

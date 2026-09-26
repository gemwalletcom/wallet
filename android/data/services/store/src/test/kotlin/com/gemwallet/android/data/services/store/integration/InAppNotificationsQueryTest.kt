package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.DbInAppNotification
import com.gemwallet.android.data.services.store.database.entities.DbWallet
import com.gemwallet.android.data.services.store.queries.InAppNotificationsQuery
import com.wallet.core.primitives.CoreListItem
import com.wallet.core.primitives.InAppNotification
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class InAppNotificationsQueryTest {
    private lateinit var database: GemDatabase
    private lateinit var query: InAppNotificationsQuery

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database = Room.inMemoryDatabaseBuilder(
            InstrumentationRegistry.getInstrumentation().targetContext,
            GemDatabase::class.java,
        ).build()
        query = InAppNotificationsQuery(database.inAppNotificationsDao())
        listOf("wallet-1", "wallet-2").forEach { id ->
            database.walletsDao().insert(DbWallet(id = id, name = id, domainName = null, type = WalletType.Multicoin, position = 0, pinned = false, index = 0, source = WalletSource.Import))
        }
        database.inAppNotificationsDao().put(
            listOf(
                notification(id = "older", walletId = "wallet-1", createdAt = 10, readAt = 15),
                notification(id = "newest", walletId = "wallet-1", createdAt = 30, readAt = null),
                notification(id = "other-wallet", walletId = "wallet-2", createdAt = 40, readAt = null),
                notification(id = "middle", walletId = "wallet-1", createdAt = 20, readAt = null),
            ),
        )
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun listsTheWalletNotificationsNewestFirst() = runBlocking(Dispatchers.IO) {
        val notifications = query("wallet-1").first()

        assertEquals(listOf("newest", "middle", "older"), notifications.map { it.item.id })
        assertEquals(
            InAppNotification(walletId = WalletId("wallet-1"), readAt = 15, createdAt = 10, item = CoreListItem(id = "older", title = "older", url = "gem://older")),
            notifications.last(),
        )
    }

    @Test
    fun walletWithoutNotificationsListsNone() = runBlocking(Dispatchers.IO) {
        assertEquals(emptyList<InAppNotification>(), query("wallet-3").first())
    }

    private fun notification(id: String, walletId: String, createdAt: Long, readAt: Long?) = DbInAppNotification(
        id = id,
        walletId = walletId,
        readAt = readAt,
        createdAt = createdAt,
        item = CoreListItem(id = id, title = id, url = "gem://$id"),
    )
}

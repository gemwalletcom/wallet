package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.queries.WalletsQuery
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
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
class WalletsQueryTest {
    private val database = Room.inMemoryDatabaseBuilder(
        InstrumentationRegistry.getInstrumentation().targetContext,
        GemDatabase::class.java,
    ).build()
    private val query = WalletsQuery(database.walletsDao())
    private val multicoin = mockWallet(
        id = "multicoin_0x1",
        name = "Main",
        accounts = listOf(mockAccount(chain = Chain.Bitcoin, address = "bc1main"), mockAccount(chain = Chain.Ethereum, address = "0xmain")),
    )
    private val pinned = mockWallet(id = "single_ethereum_0x2", name = "Pinned", type = WalletType.Single, accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xpinned")))
        .copy(isPinned = true)
    private val watch = mockWallet(id = "view_ethereum_0x3", name = "Watch", type = WalletType.View)

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database.assetsDao().insert(listOf(Chain.Bitcoin, Chain.Ethereum).map { mockAsset(id = mockAssetId(chain = it)).toRecord() })
        listOf(multicoin, pinned, watch).forEach { wallet ->
            database.walletsDao().insert(wallet.toRecord())
            database.accountsDao().insert(wallet.accounts.map { it.toRecord(wallet.id.id) })
        }
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun everyWalletComesWithItsOwnAccountsAndPinState() = runBlocking(Dispatchers.IO) {
        val wallets = query().first().associateBy { it.id }

        assertEquals(multicoin, wallets[multicoin.id])
        assertEquals(pinned, wallets[pinned.id])
    }

    @Test
    fun aWalletWithoutAccountsIsStillListed() = runBlocking(Dispatchers.IO) {
        assertEquals(watch, query().first().firstOrNull { it.id == watch.id })
    }
}

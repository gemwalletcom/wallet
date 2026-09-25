package com.gemwallet.android.data.services.store.integration

import androidx.room.Room
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.queries.WalletQuery
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletId
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
class WalletQueryTest {
    private val database = Room.inMemoryDatabaseBuilder(
        InstrumentationRegistry.getInstrumentation().targetContext,
        GemDatabase::class.java,
    ).build()
    private val query = WalletQuery(database.walletsDao(), database.accountsDao())
    private val wallet = mockWallet(
        id = "multicoin_0x1",
        name = "Main",
        accounts = listOf(mockAccount(chain = Chain.Bitcoin, address = "bc1main"), mockAccount(chain = Chain.Ethereum, address = "0xmain")),
    )
    private val otherWallet = mockWallet(id = "multicoin_0x2", name = "Other", accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xother")))

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database.assetsDao().insert(listOf(Chain.Bitcoin, Chain.Ethereum).map { mockAsset(id = mockAssetId(chain = it)).toRecord() })
        listOf(wallet, otherWallet).forEach { wallet ->
            database.walletsDao().insert(wallet.toRecord())
            database.accountsDao().insert(wallet.accounts.map { it.toRecord(wallet.id.id) })
        }
    }

    @After
    fun tearDown() = database.close()

    @Test
    fun theWalletIsFoundByIdWithOnlyItsOwnAccounts() = runBlocking(Dispatchers.IO) {
        assertEquals(wallet, query(wallet.id).first())
    }

    @Test
    fun anUnknownIdFindsNoWallet() = runBlocking(Dispatchers.IO) {
        assertNull(query(WalletId("multicoin_0x3")).first())
    }
}

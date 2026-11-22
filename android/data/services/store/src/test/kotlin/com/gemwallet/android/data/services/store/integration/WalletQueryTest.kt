package com.gemwallet.android.data.services.store.integration

import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.data.services.store.database.mockGemDatabase
import com.gemwallet.android.data.services.store.queries.WalletQuery
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import kotlin.time.Duration.Companion.seconds

@RunWith(AndroidJUnit4::class)
class WalletQueryTest {
    private val database = mockGemDatabase()
    private val query = WalletQuery(database.walletsDao())
    private val wallet = mockWallet(
        id = WalletId("multicoin_0x1"),
        name = "Main",
        accounts = listOf(mockAccount(chain = Chain.Bitcoin, address = "bc1main"), mockAccount(chain = Chain.Ethereum, address = "0xmain")),
    )
    private val otherWallet = mockWallet(id = WalletId("multicoin_0x2"), name = "Other", accounts = listOf(mockAccount(chain = Chain.Ethereum, address = "0xother")))

    @Before
    fun setUp() = runBlocking(Dispatchers.IO) {
        database.assetsDao().insert(listOf(Chain.Bitcoin, Chain.Ethereum, Chain.Solana).map { mockAsset(id = mockAssetId(chain = it)).toRecord() })
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

    @Test
    fun anAccountAddedToTheWalletIsEmitted() = runBlocking(Dispatchers.IO) {
        val emissions = Channel<Wallet?>(Channel.UNLIMITED)
        val observer = launch { query(wallet.id).collect(emissions::send) }
        assertEquals(wallet, emissions.receive())

        val account = mockAccount(chain = Chain.Solana, address = "solmain")
        database.walletsDao().insert(wallet.toRecord())
        database.accountsDao().insert(listOf(account.toRecord(wallet.id.id)))

        assertEquals(wallet.copy(accounts = wallet.accounts + account), withTimeout(5.seconds) { emissions.receive() })
        observer.cancel()
    }
}

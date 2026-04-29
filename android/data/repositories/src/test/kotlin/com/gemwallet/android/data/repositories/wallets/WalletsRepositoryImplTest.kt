package com.gemwallet.android.data.repositories.wallets

import com.gemwallet.android.application.wallet.coordinators.WalletIdGenerator
import com.gemwallet.android.blockchain.operators.CreateAccountOperator
import com.gemwallet.android.data.service.store.database.AccountsDao
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.WalletsDao
import com.gemwallet.android.data.service.store.database.entities.DbAccount
import com.gemwallet.android.data.service.store.database.entities.DbAsset
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.isStakeSupported
import com.gemwallet.android.ext.isSwapSupport
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.WalletType
import io.mockk.coEvery
import io.mockk.coJustRun
import io.mockk.coVerifyOrder
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.assetDefaultRank

class WalletsRepositoryImplTest {

    private val walletsDao = mockk<WalletsDao>(relaxed = true)
    private val accountsDao = mockk<AccountsDao>(relaxed = true)
    private val assetsDao = mockk<AssetsDao>(relaxed = true)
    private val createAccount = mockk<CreateAccountOperator>(relaxed = true)
    private val walletIdGenerator = mockk<WalletIdGenerator>(relaxed = true)
    private val transactionRunner = RecordingStoreTransactionRunner()

    private val subject = WalletsRepositoryImpl(
        walletsDao = walletsDao,
        accountsDao = accountsDao,
        assetsDao = assetsDao,
        createAccount = createAccount,
        walletIdGenerator = walletIdGenerator,
        transactionRunner = transactionRunner,
    )

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun addWatch_insertsNativeAssetBeforeAccount() = runBlocking {
        stubNativeAssets()
        every { walletIdGenerator.generateWalletId(WalletType.View, Chain.Ethereum, "0xabc") } returns "wallet-1"
        every { walletsDao.getById("wallet-1") } returns flowOf(null)
        every { walletsDao.getAll() } returns flowOf(emptyMap())
        coEvery { accountsDao.getByWalletId("wallet-1") } returns emptyList()
        coJustRun { walletsDao.insert(any()) }
        coJustRun { assetsDao.insert(any<List<DbAsset>>()) }
        coJustRun { accountsDao.insert(any<List<DbAccount>>()) }

        subject.addWatch("Wallet", "0xabc", Chain.Ethereum)

        coVerifyOrder {
            walletsDao.insert(any())
            assetsDao.insert(match<List<DbAsset>> { records -> records.map { it.id } == listOf("ethereum") })
            accountsDao.insert(any<List<DbAccount>>())
        }
        assertEquals(1, transactionRunner.runCount)
    }

    @Test
    fun updateAccounts_insertsNativeAssetsBeforeAccounts() = runBlocking {
        stubNativeAssets()
        val wallet = mockWallet(
            id = "wallet-1",
            accounts = listOf(
                mockAccount(chain = Chain.Ethereum),
                mockAccount(chain = Chain.Solana),
            )
        )
        coJustRun { assetsDao.insert(any<List<DbAsset>>()) }
        coJustRun { accountsDao.insert(any<List<DbAccount>>()) }

        subject.updateAccounts(wallet)

        coVerifyOrder {
            assetsDao.insert(match<List<DbAsset>> { records ->
                records.map { it.id }.toSet() == setOf("ethereum", "solana")
            })
            accountsDao.insert(any<List<DbAccount>>())
        }
        assertEquals(1, transactionRunner.runCount)
    }

    private fun stubNativeAssets() {
        mockkStatic("com.gemwallet.android.ext.ChainKt")
        mockkStatic("uniffi.gemstone.GemstoneKt")

        every { Chain.Ethereum.asset() } returns mockAsset(chain = Chain.Ethereum, name = "Ethereum", symbol = "ETH", decimals = 18)
        every { Chain.Solana.asset() } returns mockAsset(chain = Chain.Solana, name = "Solana", symbol = "SOL", decimals = 9)
        every { Chain.Ethereum.isSwapSupport() } returns true
        every { Chain.Solana.isSwapSupport() } returns true
        every { Chain.Ethereum.isStakeSupported() } returns false
        every { Chain.Solana.isStakeSupported() } returns true
        every { assetDefaultRank(any()) } returns 1
    }

    private class RecordingStoreTransactionRunner : StoreTransactionRunner {
        var runCount = 0

        override suspend fun <T> run(block: suspend () -> T): T {
            runCount += 1
            return block()
        }
    }
}

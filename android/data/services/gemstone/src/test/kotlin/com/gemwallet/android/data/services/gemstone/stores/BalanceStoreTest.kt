package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.BalancesDao
import com.gemwallet.android.data.services.store.database.entities.DbBalance
import com.gemwallet.android.data.services.store.database.mockStoreTransactionRunner
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.Chain
import io.mockk.coVerify
import io.mockk.mockk
import io.mockk.slot
import io.mockk.verify
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemBalanceRecord
import uniffi.gemstone.GemBalanceValue
import java.math.BigInteger

class GemstoneBalanceStoreTest {

    private val balancesDao = mockk<BalancesDao>(relaxed = true)
    private val assetsDao = mockk<AssetsDao>(relaxed = true)
    private val subject = GemstoneBalanceStore(balancesDao, assetsDao, mockStoreTransactionRunner())

    @Test
    fun addBalancesInsertsEveryRowInOneStatementWithCoresFlag() = runTest {
        val solana = mockAssetId(chain = Chain.Solana)
        val ethereum = mockAssetId(chain = Chain.Ethereum)
        val walletId = mockWalletId()

        subject.addBalances(walletId.id, listOf(solana.toIdentifier(), ethereum.toIdentifier()), false)

        val balances = slot<List<DbBalance>>()
        coVerify(exactly = 1) { assetsDao.insertBalances(capture(balances)) }
        assertEquals(listOf(solana.toIdentifier(), ethereum.toIdentifier()), balances.captured.map { it.assetId })
        assertEquals(listOf(walletId.id, walletId.id), balances.captured.map { it.walletId })
        assertEquals(listOf(false, false), balances.captured.map { it.isVisible })
    }

    @Test
    fun balanceWritesTheWholeRowWithoutCreatingIt() = runTest {
        val zero = GemBalanceValue(BigInteger.ZERO, 0.0)
        subject.updateBalances(
            "wallet-1",
            listOf(
                GemBalanceRecord(
                    assetId = "ethereum_0xtoken",
                    available = GemBalanceValue(BigInteger("1000000000000000000"), 1.0),
                    frozen = zero,
                    locked = zero,
                    staked = zero,
                    pending = zero,
                    pendingUnconfirmed = zero,
                    rewards = zero,
                    reserved = zero,
                    withdrawable = zero,
                    earn = zero,
                    metadata = null,
                    isActive = true,
                ),
            ),
        )

        verify(exactly = 0) { balancesDao.insertIgnore(any()) }
        verify {
            balancesDao.updateBalance(
                walletId = "wallet-1",
                assetId = "ethereum_0xtoken",
                available = "1000000000000000000",
                availableAmount = 1.0,
                frozen = "0", frozenAmount = 0.0, locked = "0", lockedAmount = 0.0, staked = "0", stakedAmount = 0.0,
                pending = "0", pendingAmount = 0.0, pendingUnconfirmed = "0", pendingUnconfirmedAmount = 0.0,
                rewards = "0", rewardsAmount = 0.0, reserved = "0", reservedAmount = 0.0, withdrawable = "0", withdrawableAmount = 0.0,
                earn = "0", earnAmount = 0.0,
                metadata = null,
                isActive = true,
                updatedAt = any(),
            )
        }
    }
}

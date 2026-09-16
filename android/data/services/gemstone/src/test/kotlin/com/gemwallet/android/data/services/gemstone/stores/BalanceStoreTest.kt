package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.mockStoreTransactionRunner
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.test.runTest
import org.junit.Test
import uniffi.gemstone.GemBalanceRecord
import uniffi.gemstone.GemBalanceValue
import java.math.BigInteger

class GemstoneBalanceStoreTest {

    private val balancesDao = mockk<BalancesDao>(relaxed = true)
    private val subject = GemstoneBalanceStore(balancesDao, mockk<AssetsDao>(relaxed = true), mockStoreTransactionRunner())

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
                votes = 0L, energyAvailable = 0L, energyTotal = 0L, bandwidthAvailable = 0L, bandwidthTotal = 0L,
                isActive = true,
                updatedAt = any(),
            )
        }
    }
}

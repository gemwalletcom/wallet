package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.mockStoreTransactionRunner
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.test.runTest
import org.junit.Test
import uniffi.gemstone.GemBalanceUpdate
import uniffi.gemstone.GemBalanceUpdateType
import uniffi.gemstone.GemBalanceValue

class GemstoneBalanceStoreTest {

    private val balancesDao = mockk<BalancesDao>(relaxed = true)
    private val subject = GemstoneBalanceStore(balancesDao, mockk<AssetsDao>(relaxed = true), mockStoreTransactionRunner())

    @Test
    fun tokenUpdateWritesBalanceWithoutCreatingRows() = runTest {
        subject.updateBalances(
            "wallet-1",
            listOf(GemBalanceUpdate("ethereum_0xtoken", GemBalanceUpdateType.Token(GemBalanceValue("1000000000000000000", 1.0)), true)),
        )

        verify(exactly = 0) { balancesDao.insertIgnore(any()) }
        verify { balancesDao.updateTokenBalance("wallet-1", "ethereum_0xtoken", "1000000000000000000", 1.0, true, any()) }
    }
}

package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import io.mockk.mockk
import io.mockk.slot
import io.mockk.verify
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test
import uniffi.gemstone.GemBalanceUpdate
import uniffi.gemstone.GemBalanceUpdateType
import uniffi.gemstone.GemBalanceValue

class GemstoneBalanceStoreTest {

    private val balancesDao = mockk<BalancesDao>(relaxed = true)
    private val subject = GemstoneBalanceStore(balancesDao)

    @Test
    fun tokenUpdateCreatesHiddenRowBeforeWritingBalance() = runTest {
        val inserted = slot<DbBalance>()

        subject.updateBalances(
            "wallet-1",
            listOf(GemBalanceUpdate("ethereum_0xtoken", GemBalanceUpdateType.Token(GemBalanceValue("1000000000000000000", 1.0)), true)),
        )

        verify { balancesDao.insertIgnore(capture(inserted)) }
        assertEquals("ethereum_0xtoken", inserted.captured.assetId)
        assertEquals("wallet-1", inserted.captured.walletId)
        assertFalse(inserted.captured.isVisible)
        verify { balancesDao.updateTokenBalance("wallet-1", "ethereum_0xtoken", "1000000000000000000", 1.0, true, any()) }
    }
}

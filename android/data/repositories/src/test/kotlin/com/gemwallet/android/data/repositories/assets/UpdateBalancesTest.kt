package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.blockchain.services.BalancesService
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.ext.asset
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetEthereum
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkAll
import io.mockk.verify
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class UpdateBalancesTest {

    private val balancesDao = mockk<BalancesDao>(relaxed = true)
    private val balancesService = mockk<BalancesService>(relaxed = true)

    private val subject = UpdateBalances(
        balancesDao = balancesDao,
        balancesService = balancesService,
    )

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun `failed native refresh does not create empty balance row`() = runTest {
        val walletId = "wallet-1"
        val account = mockAccount(chain = Chain.Ethereum)

        every { balancesDao.getByAccount(walletId, account.address, account.chain.string) } returns null
        coEvery { balancesService.getNativeBalances(account) } returns null
        coEvery { balancesService.getStakeBalances(account) } returns null

        val result = subject.updateBalances(walletId, account, emptyList())

        assertTrue(result.isEmpty())
        verify(exactly = 0) { balancesDao.insertIgnore(any()) }
        verify(exactly = 0) {
            balancesDao.updateCoinBalance(any(), any(), any(), any(), any(), any(), any(), any(), any())
        }
    }

    @Test
    fun `failed native refresh keeps previous balance row`() = runTest {
        val walletId = "wallet-1"
        val account = mockAccount(chain = Chain.Ethereum)
        val existing = DbBalance(
            assetId = account.chain.string,
            walletId = walletId,
            accountAddress = account.address,
            available = "1000000000000000000",
            availableAmount = 1.0,
            updatedAt = 1L,
        )

        mockkStatic("com.gemwallet.android.ext.ChainKt")
        every { Chain.Ethereum.asset() } returns mockAssetEthereum()
        every { balancesDao.getByAccount(walletId, account.address, account.chain.string) } returns existing
        coEvery { balancesService.getNativeBalances(account) } returns null
        coEvery { balancesService.getStakeBalances(account) } returns null

        val result = subject.updateBalances(walletId, account, emptyList())

        assertEquals(1, result.size)
        assertEquals("1000000000000000000", result.single().balance.available)
        verify(exactly = 0) { balancesDao.insertIgnore(any()) }
        verify(exactly = 0) {
            balancesDao.updateCoinBalance(any(), any(), any(), any(), any(), any(), any(), any(), any())
        }
    }
}

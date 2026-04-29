package com.gemwallet.android.data.repositories.assets

import com.gemwallet.android.blockchain.services.BalancesService
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetEthereum
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.slot
import io.mockk.unmockkAll
import io.mockk.verify
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
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

        every { balancesDao.getByAsset(walletId, account.chain.string) } returns null
        coEvery { balancesService.getNativeBalances(account) } returns null
        coEvery { balancesService.getStakeBalances(account) } returns null

        val result = subject.updateBalances(walletId, account, emptyList())

        assertTrue(result.isEmpty())
        verify(exactly = 0) { balancesDao.insertIgnore(any()) }
        verify(exactly = 0) {
            balancesDao.updateCoinBalance(any(), any(), any(), any(), any(), any(), any(), any())
        }
    }

    @Test
    fun `failed native refresh keeps previous balance row`() = runTest {
        val walletId = "wallet-1"
        val account = mockAccount(chain = Chain.Ethereum)
        val existing = DbBalance(
            assetId = account.chain.string,
            walletId = walletId,
            available = "1000000000000000000",
            availableAmount = 1.0,
            updatedAt = 1L,
        )

        mockkStatic("com.gemwallet.android.ext.ChainKt")
        every { Chain.Ethereum.asset() } returns mockAssetEthereum()
        every { balancesDao.getByAsset(walletId, account.chain.string) } returns existing
        coEvery { balancesService.getNativeBalances(account) } returns null
        coEvery { balancesService.getStakeBalances(account) } returns null

        val result = subject.updateBalances(walletId, account, emptyList())

        assertEquals(1, result.size)
        assertEquals("1000000000000000000", result.single().balance.available)
        verify(exactly = 0) { balancesDao.insertIgnore(any()) }
        verify(exactly = 0) {
            balancesDao.updateCoinBalance(any(), any(), any(), any(), any(), any(), any(), any())
        }
    }

    @Test
    fun `token refresh creates hidden balance row`() = runTest {
        val walletId = "wallet-1"
        val account = mockAccount(chain = Chain.Ethereum)
        val token = mockAsset(
            chain = Chain.Ethereum,
            tokenId = "0xtoken",
            name = "Token",
            symbol = "TOK",
            decimals = 18,
            type = AssetType.ERC20,
        )
        val inserted = slot<DbBalance>()

        every { balancesDao.getByAsset(walletId, account.chain.string) } returns null
        coEvery { balancesService.getNativeBalances(account) } returns null
        coEvery { balancesService.getStakeBalances(account) } returns null
        coEvery { balancesService.getTokensBalances(account, listOf(token)) } returns listOf(
            AssetBalance.create(token, available = "1000000000000000000")
        )

        subject.updateBalances(walletId, account, listOf(token))

        verify { balancesDao.insertIgnore(capture(inserted)) }
        assertEquals(token.id.toIdentifier(), inserted.captured.assetId)
        assertEquals(walletId, inserted.captured.walletId)
        assertFalse(inserted.captured.isVisible)
    }
}

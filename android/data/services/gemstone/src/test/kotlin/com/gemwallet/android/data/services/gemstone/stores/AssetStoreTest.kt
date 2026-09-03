package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.entities.DbAsset
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockWalletId
import io.mockk.coVerify
import io.mockk.mockk
import io.mockk.slot
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test

class GemstoneAssetStoreTest {

    private val assetsDao = mockk<AssetsDao>(relaxed = true)
    private val subject = GemstoneAssetStore(assetsDao)

    @Test
    fun addMissingBalances_insertsHiddenBalancesInOneStatement() = runBlocking {
        val solana = mockAssetSolana()
        val ethereum = mockAssetEthereum()
        val walletId = mockWalletId()

        subject.addMissingBalances(walletId.id, listOf(solana.id.toIdentifier(), ethereum.id.toIdentifier()))

        val balances = slot<List<DbBalance>>()
        coVerify(exactly = 1) { assetsDao.insertBalances(capture(balances)) }
        assertEquals(listOf(solana.id.toIdentifier(), ethereum.id.toIdentifier()), balances.captured.map { it.assetId })
        assertEquals(listOf(walletId.id, walletId.id), balances.captured.map { it.walletId })
        assertEquals(listOf(false, false), balances.captured.map { it.isVisible })
    }

    @Test
    fun addBalances_insertsMissingBalanceWithoutTouchingAssets() = runBlocking {
        val walletId = mockWalletId()

        subject.addBalances(walletId.id, listOf("bitcoin"), true)

        coVerify(exactly = 0) { assetsDao.insert(any<DbAsset>()) }
        coVerify { assetsDao.insertBalances(match { it.single().assetId == "bitcoin" && it.single().walletId == walletId.id && it.single().isVisible }) }
        coVerify(exactly = 0) { assetsDao.setWalletAssetVisibility(any(), any(), any()) }
    }
}

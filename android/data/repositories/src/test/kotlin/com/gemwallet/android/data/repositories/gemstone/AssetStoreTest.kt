package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.repositories.assets.AssetsAvailabilityService
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockWalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import io.mockk.slot
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test

class GemstoneAssetStoreTest {

    private val assetsDao = mockk<AssetsDao>(relaxed = true)
    private val subject = GemstoneAssetStore(assetsDao, AssetsAvailabilityService(assetsDao))

    @Test
    fun addMissingBalances_insertsHiddenBalanceOnlyForExistingAssets() = runBlocking {
        val present = mockAssetSolana()
        val absent = mockAssetEthereum()
        val walletId = mockWalletId()
        coEvery {
            assetsDao.getAssetIds(listOf(present.id.toIdentifier(), absent.id.toIdentifier()))
        } returns listOf(present.id.toIdentifier())

        subject.addMissingBalances(walletId.id, listOf(present.id.toIdentifier(), absent.id.toIdentifier()))

        val balanceSlot = slot<DbBalance>()
        coVerify(exactly = 1) { assetsDao.insertBalance(capture(balanceSlot)) }
        assertEquals(present.id.toIdentifier(), balanceSlot.captured.assetId)
        assertEquals(walletId.id, balanceSlot.captured.walletId)
        assertEquals(false, balanceSlot.captured.isVisible)
    }
}

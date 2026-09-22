package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.entities.mockDbAssetInfo
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockWalletId
import io.mockk.coEvery
import io.mockk.mockk
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import java.math.BigInteger

class GemstonePortfolioStoreTest {

    @Test
    fun `hands core the stored balance instead of a total the app summed`() = runBlocking {
        val bitcoin = mockAsset()
        val walletId = mockWalletId()
        val assetsDao = mockk<AssetsDao> {
            coEvery { getPortfolioAssets(walletId.id) } returns listOf(
                mockDbAssetInfo(
                    chain = bitcoin.id.chain,
                    walletId = walletId.id,
                    balanceAvailable = "1000",
                ),
            )
        }
        val subject = GemstonePortfolioStore(assetsDao)

        val balances = subject.getWalletBalances(walletId.id)

        assertEquals(listOf(bitcoin.id.toIdentifier()), balances.map { it.assetId })
        assertEquals(listOf(BigInteger("1000")), balances.map { it.available })
    }
}

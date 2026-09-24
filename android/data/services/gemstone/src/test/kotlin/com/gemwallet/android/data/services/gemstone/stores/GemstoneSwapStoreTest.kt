package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.AssetsRequestFilter
import com.gemwallet.android.data.service.store.database.TransactionsDao
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.RecentActivityType
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.Test

class GemstoneSwapStoreTest {

    private val assetsDao = mockk<AssetsDao>(relaxed = true)
    private val subject = GemstoneSwapStore(assetsDao, mockk<TransactionsDao>(relaxed = true))

    @Test
    fun `pay candidates ask the database for one enabled swappable page`() = runBlocking {
        every { assetsDao.search(any(), any(), any(), any(), any(), any(), any(), any(), any(), any(), any(), any(), any(), any()) } returns flowOf(emptyList())

        subject.getPayAssetIds(mockWalletId().id, 25u)

        verify {
            assetsDao.search(
                walletId = mockWalletId().id,
                query = "",
                limit = 25,
                enabled = true,
                swappable = true,
            )
        }
    }

    @Test
    fun `receive candidates keep the same eligibility inside the supported chains`() = runBlocking {
        every { assetsDao.search(any(), any(), any(), any(), any(), any(), any(), any(), any(), any(), any(), any(), any(), any()) } returns flowOf(emptyList())

        subject.getReceiveAssetIds(mockWalletId().id, listOf("ethereum"), listOf("ethereum"), 25u)

        verify {
            assetsDao.search(
                walletId = mockWalletId().id,
                query = "",
                limit = 25,
                enabled = true,
                swappable = true,
                byChainsOrAssetIds = true,
                chains = any(),
                assetIds = listOf("ethereum"),
            )
        }
    }

    @Test
    fun `recent candidates keep the swap history enabled swappable and capped`() = runBlocking {
        every { assetsDao.getRecentAssets(any(), any(), any(), any()) } returns flowOf(emptyList())

        subject.getRecentAssetIds(mockWalletId().id, 20u)

        verify {
            assetsDao.getRecentAssets(
                walletId = mockWalletId().id,
                type = listOf(RecentActivityType.SwapSelect, RecentActivityType.Swap),
                filters = setOf(AssetsRequestFilter.Enabled, AssetsRequestFilter.Swappable),
                limit = 20,
            )
        }
    }
}

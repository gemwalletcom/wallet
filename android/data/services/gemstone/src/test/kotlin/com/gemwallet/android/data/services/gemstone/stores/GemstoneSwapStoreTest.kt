package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.AssetsDao
import com.gemwallet.android.data.services.store.database.AssetsRequestFilter
import com.gemwallet.android.data.services.store.database.TransactionsDao
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.Test
import uniffi.gemstone.GemAssetFilter
import uniffi.gemstone.RecentActivityType
import com.wallet.core.primitives.RecentActivityType as PrimitiveRecentActivityType

class GemstoneSwapStoreTest {

    private val assetsDao = mockk<AssetsDao>(relaxed = true)
    private val subject = GemstoneSwapStore(assetsDao, mockk<TransactionsDao>(relaxed = true))

    @Test
    fun `asset candidates ask the database for one page with the filters Core names`() = runBlocking {
        every { assetsDao.search(any(), any(), any(), any(), any(), any(), any(), any(), any(), any(), any(), any(), any(), any(), any()) } returns flowOf(emptyList())

        subject.getAssetIds(
            mockWalletId().id,
            listOf(GemAssetFilter.Enabled, GemAssetFilter.Swappable, GemAssetFilter.ChainsOrAssetIds(listOf("ethereum"), listOf("ethereum"))),
            25u,
        )

        verify {
            assetsDao.search(
                walletId = mockWalletId().id,
                query = "",
                limit = 25,
                exclude = emptyList(),
                enabled = true,
                buyable = false,
                sellable = false,
                swappable = true,
                hasBalance = false,
                hasAvailableBalance = false,
                byChainsOrAssetIds = true,
                chains = listOf(Chain.Ethereum),
                assetIds = listOf("ethereum"),
                byChains = false,
                selectedChains = emptyList(),
            )
        }
    }

    @Test
    fun `recent candidates ask the database with the activity types and filters Core names`() = runBlocking {
        every { assetsDao.getRecentAssets(any(), any(), any(), any()) } returns flowOf(emptyList())

        subject.getRecentAssetIds(
            mockWalletId().id,
            listOf(RecentActivityType.SWAP_SELECT, RecentActivityType.SWAP),
            listOf(GemAssetFilter.Enabled, GemAssetFilter.Swappable),
            20u,
        )

        verify {
            assetsDao.getRecentAssets(
                walletId = mockWalletId().id,
                type = listOf(PrimitiveRecentActivityType.SwapSelect, PrimitiveRecentActivityType.Swap),
                filters = setOf(AssetsRequestFilter.Enabled, AssetsRequestFilter.Swappable),
                limit = 20,
            )
        }
    }
}

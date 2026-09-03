package com.gemwallet.android.data.services.gemstone.assets

import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.service.store.database.AssetListDao
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.SearchDao
import com.gemwallet.android.data.service.store.database.entities.mockDbAssetInfo
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAssetSolana
import com.gemwallet.android.testkit.mockAssetSolanaUSDC
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import io.mockk.every
import io.mockk.mockk
import io.mockk.unmockkAll
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Test

class AssetsSearchServiceTest {
    private val assetsDao = mockk<AssetsDao>(relaxed = true)
    private val searchDao = mockk<SearchDao>(relaxed = true)
    private val assetListDao = mockk<AssetListDao>(relaxed = true)
    private val getCurrentWalletId = mockk<GetCurrentWalletId>()

    @After
    fun tearDown() {
        unmockkAll()
    }

    @Test
    fun swapSearch_includesEnabledHiddenAndUnlinkedAssets() = runBlocking {
        every { searchDao.hasAssetPriorities("") } returns flowOf(0)

        val wallet = mockWallet(
            id = "wallet-1",
            accounts = listOf(mockAccount(chain = Chain.Solana)),
        )
        val enabledAsset = mockAssetSolana()
        val hiddenAsset = mockAssetSolanaUSDC()
        val unlinkedAsset = mockAssetSolanaUSDC().copy(
            id = AssetId(Chain.Solana, "jto"),
            name = "Jito",
            symbol = "JTO",
            decimals = 9,
        )
        val disabledAsset = mockAssetSolanaUSDC().copy(
            id = AssetId(Chain.Solana, "bonk"),
            name = "Bonk",
            symbol = "BONK",
            decimals = 5,
        )

        every {
            assetsDao.swapSearch(
                walletId = "wallet-1",
                query = "",
                byChains = listOf(Chain.Solana),
                byAssets = emptyList(),
            )
        } returns flowOf(
            listOf(
                mockDbAssetInfo(asset = enabledAsset, walletId = "wallet-1", visible = true),
                mockDbAssetInfo(asset = hiddenAsset, walletId = "wallet-1", visible = false),
                mockDbAssetInfo(
                    asset = unlinkedAsset,
                    walletId = null,
                    visible = false,
                    address = null,
                ),
                mockDbAssetInfo(
                    asset = disabledAsset,
                    walletId = "wallet-1",
                    visible = true,
                    assetRank = -1,
                ),
            )
        )

        val subject = AssetsSearchService(assetsDao, searchDao, assetListDao, getCurrentWalletId)
        val result = subject.swapSearch(
            wallet = wallet,
            query = "",
            byChains = listOf(Chain.Solana),
            byAssets = emptyList(),
        ).first()

        assertEquals(listOf(enabledAsset.id, hiddenAsset.id, unlinkedAsset.id), result.map { it.asset.id })
    }

    @Test
    fun swapSearch_usesPriorityDaoAndPreservesOrderWhenPrioritiesExist() = runBlocking {
        every { searchDao.hasAssetPriorities("usd") } returns flowOf(2)

        val wallet = mockWallet(
            id = "wallet-1",
            accounts = listOf(mockAccount(chain = Chain.Solana)),
        )
        val highPriorityAsset = mockAssetSolana()
        val lowPriorityAsset = mockAssetSolanaUSDC()

        every {
            assetsDao.swapSearchWithPriority(
                walletId = "wallet-1",
                query = "usd",
                byChains = listOf(Chain.Solana),
                byAssets = emptyList(),
            )
        } returns flowOf(
            listOf(
                mockDbAssetInfo(asset = highPriorityAsset, walletId = "wallet-1", visible = true),
                mockDbAssetInfo(asset = lowPriorityAsset, walletId = "wallet-1", visible = true),
            )
        )

        val subject = AssetsSearchService(assetsDao, searchDao, assetListDao, getCurrentWalletId)
        val result = subject.swapSearch(
            wallet = wallet,
            query = "usd",
            byChains = listOf(Chain.Solana),
            byAssets = emptyList(),
        ).first()

        assertEquals(listOf(highPriorityAsset.id, lowPriorityAsset.id), result.map { it.asset.id })
    }
}

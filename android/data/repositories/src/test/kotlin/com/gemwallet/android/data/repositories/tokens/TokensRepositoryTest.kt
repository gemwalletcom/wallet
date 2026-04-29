package com.gemwallet.android.data.repositories.tokens

import com.gemwallet.android.application.assets.coordinators.SearchAssets
import com.gemwallet.android.blockchain.services.TokenService
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.AssetsPriorityDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.DbAssetBasicUpdate
import com.gemwallet.android.data.service.store.database.entities.DbAssetPriority
import com.gemwallet.android.data.service.store.database.entities.DbFiatRate
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetBasic
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.slot
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class TokensRepositoryTest {

    private val assetsDao = mockk<AssetsDao>(relaxed = true)
    private val pricesDao = mockk<PricesDao>(relaxed = true)
    private val assetsPriorityDao = mockk<AssetsPriorityDao>(relaxed = true)
    private val searchAssets = mockk<SearchAssets>()
    private val tokenService = mockk<TokenService>(relaxed = true)

    private val subject = TokensRepository(
        assetsDao = assetsDao,
        pricesDao = pricesDao,
        assetsPriorityDao = assetsPriorityDao,
        searchAssets = searchAssets,
        tokenService = tokenService,
    )

    @Test
    fun search_usesSearchAssetsAndStoresPriority() = runTest {
        val asset = mockAssetBasic()
        coEvery {
            searchAssets.search(
                query = "btc",
                chains = listOf(Chain.Bitcoin),
                tags = listOf(AssetTag.Trending),
            )
        } returns listOf(asset)
        every { pricesDao.getRates(Currency.USD) } returns flowOf(DbFiatRate(Currency.USD, 1.0))

        val result = subject.search(
            query = "btc",
            currency = Currency.USD,
            chains = listOf(Chain.Bitcoin),
            tags = listOf(AssetTag.Trending),
        )
        val priorities = slot<List<DbAssetPriority>>()

        assertTrue(result)
        coVerify {
            searchAssets.search(
                query = "btc",
                chains = listOf(Chain.Bitcoin),
                tags = listOf(AssetTag.Trending),
            )
        }
        coVerify { assetsPriorityDao.put(capture(priorities)) }
        assertEquals("btc::trending", priorities.captured.single().query)
    }

    @Test
    fun searchByAssetIds_usesSearchAssetsGetAssets() = runTest {
        val asset = mockAsset()
        val assetBasic = mockAssetBasic(asset = asset, rank = 100)
        val updates = slot<List<DbAssetBasicUpdate>>()
        coEvery { searchAssets.getAssets(listOf(asset.id)) } returns listOf(assetBasic)
        every { pricesDao.getRates(Currency.USD) } returns flowOf(DbFiatRate(Currency.USD, 1.0))

        val result = subject.search(
            assetIds = listOf(asset.id),
            currency = Currency.USD,
        )

        assertTrue(result)
        coVerify { searchAssets.getAssets(listOf(asset.id)) }
        coVerify { assetsDao.updateBasicAssets(capture(updates)) }
        assertEquals(100, updates.captured.single().rank)
    }
}

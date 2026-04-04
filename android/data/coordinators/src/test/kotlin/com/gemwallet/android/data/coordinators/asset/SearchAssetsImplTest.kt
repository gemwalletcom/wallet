package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.gemwallet.android.testkit.mockAssetBasic
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class SearchAssetsImplTest {

    private val gemApiClient = mockk<GemApiClient>()

    private val subject = SearchAssetsImpl(
        gemApiClient = gemApiClient,
    )

    @Test
    fun search_formatsChainsAndTagsForGemApi() = runTest {
        val asset = mockAssetBasic()
        coEvery {
            gemApiClient.search(
                query = "usd",
                chains = "bitcoin,ethereum",
                tags = "trending,stablecoins",
            )
        } returns listOf(asset)

        val result = subject.search(
            query = "usd",
            chains = listOf(Chain.Bitcoin, Chain.Ethereum),
            tags = listOf(AssetTag.Trending, AssetTag.Stablecoins),
        )

        assertEquals(listOf(asset), result)
    }

    @Test
    fun getAssets_delegatesToGemApi() = runTest {
        val asset = mockAssetBasic()
        val assetIds = listOf(asset.asset.id)
        coEvery { gemApiClient.getAssets(assetIds) } returns listOf(asset)

        val result = subject.getAssets(assetIds)

        assertEquals(listOf(asset), result)
        coVerify { gemApiClient.getAssets(assetIds) }
    }
}

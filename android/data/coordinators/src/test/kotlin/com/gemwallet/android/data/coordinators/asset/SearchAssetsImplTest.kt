package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.domains.search.WalletSearchTag
import com.gemwallet.android.testkit.mockAssetBasic
import com.gemwallet.android.testkit.mockSearchResponse
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemAssetsService

class SearchAssetsImplTest {

    private val assetsService = mockk<GemAssetsService>()

    private val subject = SearchAssetsImpl(
        assetsService = assetsService,
    )

    @Test
    fun search_mapsListScopeToWireTag() = runTest {
        val response = mockSearchResponse(assets = listOf(mockAssetBasic()))
        coEvery {
            assetsService.search(query = "", chains = emptyList(), tags = listOf("stocks"))
        } returns response.toJson()

        val result = subject.search(query = "", chains = emptyList(), scope = WalletSearchTag.List("stocks"))

        assertEquals(response, result)
    }

    @Test
    fun getAssets_delegatesToGemApi() = runTest {
        val asset = mockAssetBasic()
        val assetIds = listOf(asset.asset.id)
        coEvery { assetsService.getAssets(assetIds.map { it.toIdentifier() }, null) } returns listOf(asset.toJson())

        val result = subject.getAssets(assetIds)

        assertEquals(listOf(asset), result)
        coVerify { assetsService.getAssets(assetIds.map { it.toIdentifier() }, null) }
    }
}

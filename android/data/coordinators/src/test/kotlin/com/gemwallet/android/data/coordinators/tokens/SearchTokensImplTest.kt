package com.gemwallet.android.data.coordinators.tokens

import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemAssetsService

class SearchTokensImplTest {

    private val assetsService = mockk<GemAssetsService>(relaxed = true)
    private val subject = SearchTokensImpl(assetsService)

    @Test
    fun searchByAssetIds_syncsThemThroughCore() = runTest {
        val asset = mockAsset()

        val result = subject.search(assetIds = listOf(asset.id))

        assertTrue(result)
        coVerify(exactly = 1) { assetsService.syncAssets(listOf(asset.id.toIdentifier())) }
    }
}

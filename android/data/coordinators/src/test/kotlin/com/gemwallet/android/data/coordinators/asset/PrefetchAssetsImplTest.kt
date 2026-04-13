package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.services.gemapi.GemApiClient
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetBasic
import com.gemwallet.android.testkit.mockAssetEthereum
import com.wallet.core.primitives.AssetBasic
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Test

class PrefetchAssetsImplTest {

    private val gemApiClient = mockk<GemApiClient>()
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)

    private val subject = PrefetchAssetsImpl(
        gemApiClient = gemApiClient,
        assetsRepository = assetsRepository,
    )

    @Test
    fun prefetchAssets_loadsOnlyMissingAssets() = runTest {
        val bitcoin = mockAsset()
        val ethereum = mockAssetEthereum()
        val ethereumBasic = mockAssetBasic(asset = ethereum, rank = 42)
        val assetIds = listOf(bitcoin.id, ethereum.id)

        coEvery { assetsRepository.hasAssets(assetIds) } returns setOf(bitcoin.id)
        coEvery { gemApiClient.getAssets(listOf(ethereum.id)) } returns listOf(ethereumBasic)

        subject.prefetchAssets(listOf(bitcoin.id, ethereum.id, ethereum.id))

        coVerify { gemApiClient.getAssets(listOf(ethereum.id)) }
        coVerify { assetsRepository.add(listOf(ethereumBasic)) }
    }

    @Test
    fun prefetchAssets_skipsLoadWhenAllAssetsAlreadyExist() = runTest {
        val bitcoin = mockAsset()

        coEvery { assetsRepository.hasAssets(listOf(bitcoin.id)) } returns setOf(bitcoin.id)

        subject.prefetchAssets(listOf(bitcoin.id))

        coVerify(exactly = 0) { gemApiClient.getAssets(any()) }
        coVerify(exactly = 0) { assetsRepository.add(any<List<AssetBasic>>()) }
    }
}

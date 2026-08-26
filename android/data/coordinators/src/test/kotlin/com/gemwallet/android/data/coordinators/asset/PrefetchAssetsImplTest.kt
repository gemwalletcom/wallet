package com.gemwallet.android.data.coordinators.asset

import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetBasic
import com.gemwallet.android.testkit.mockAssetEthereum
import com.wallet.core.primitives.AssetBasic
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemAssetsService

class PrefetchAssetsImplTest {

    private val assetsService = mockk<GemAssetsService>()
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)

    private val subject = PrefetchAssetsImpl(
        assetsService = assetsService,
        assetsRepository = assetsRepository,
    )

    @Test
    fun prefetchAssets_loadsOnlyMissingAssets() = runTest {
        val bitcoin = mockAsset()
        val ethereum = mockAssetEthereum()
        val ethereumBasic = mockAssetBasic(asset = ethereum, rank = 42)
        val assetIds = listOf(bitcoin.id, ethereum.id)

        coEvery { assetsRepository.hasAssets(assetIds) } returns setOf(bitcoin.id)
        coEvery { assetsService.getAssets(listOf(ethereum.id).map { it.toIdentifier() }, null) } returns listOf(ethereumBasic.toJson())

        val result = subject.prefetchAssets(listOf(bitcoin.id, ethereum.id, ethereum.id))

        coVerify { assetsService.getAssets(listOf(ethereum.id).map { it.toIdentifier() }, null) }
        coVerify { assetsRepository.add(listOf(ethereumBasic)) }
        assertEquals(listOf(ethereum.id), result)
    }

    @Test
    fun prefetchAssets_skipsLoadWhenAllAssetsAlreadyExist() = runTest {
        val bitcoin = mockAsset()

        coEvery { assetsRepository.hasAssets(listOf(bitcoin.id)) } returns setOf(bitcoin.id)

        val result = subject.prefetchAssets(listOf(bitcoin.id))

        coVerify(exactly = 0) { assetsService.getAssets(any(), any()) }
        coVerify(exactly = 0) { assetsRepository.add(any<List<AssetBasic>>()) }
        assertEquals(emptyList<com.wallet.core.primitives.AssetId>(), result)
    }
}

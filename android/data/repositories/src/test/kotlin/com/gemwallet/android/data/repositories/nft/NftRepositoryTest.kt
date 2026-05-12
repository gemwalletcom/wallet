package com.gemwallet.android.data.repositories.nft

import com.gemwallet.android.data.service.store.database.NftDao
import com.gemwallet.android.data.service.store.database.entities.DbNFTAsset
import com.gemwallet.android.data.service.store.database.entities.DbNFTCollection
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTAsset
import com.wallet.core.primitives.NFTAssetData
import com.wallet.core.primitives.NFTCollection
import com.wallet.core.primitives.NFTImages
import com.wallet.core.primitives.NFTResource
import com.wallet.core.primitives.NFTType
import com.wallet.core.primitives.VerificationStatus
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class NftRepositoryTest {

    private val gemDeviceApiClient = mockk<GemDeviceApiClient>()
    private val nftDao = mockk<NftDao>()
    private val subject = NftRepository(gemDeviceApiClient, nftDao)

    @Test
    fun getListNftReadsRequestedWallet() = runTest {
        every { nftDao.getCollections("wallet-1") } returns flowOf(listOf(dbCollection("collection-1")))
        every { nftDao.getAssets("wallet-1") } returns flowOf(listOf(dbAsset("asset-1", "collection-1")))

        val result = subject.getListNft(mockWalletId("wallet-1")).first()

        assertEquals(listOf("collection-1"), result.map { it.collection.id })
        assertEquals(listOf("asset-1"), result.flatMap { it.assets }.map { it.id })
        verify { nftDao.getCollections("wallet-1") }
        verify { nftDao.getAssets("wallet-1") }
    }

    @Test
    fun getAssetNftReadsCachedAssetWithoutWalletAssociation() = runTest {
        val assetId = AssetId(Chain.Ethereum, "asset-1")
        every { nftDao.getAsset("ethereum_asset-1") } returns flowOf(dbAsset("ethereum_asset-1", "collection-2"))
        every { nftDao.getCollection("collection-2") } returns flowOf(dbCollection("collection-2"))

        val result = subject.getAssetNft(assetId).first()

        assertEquals("collection-2", result.collection.id)
        assertEquals("ethereum_asset-1", result.assets.single().id)
        verify { nftDao.getAsset("ethereum_asset-1") }
        coVerify(exactly = 0) { gemDeviceApiClient.getNFT("ethereum_asset-1") }
        verify { nftDao.getCollection("collection-2") }
    }

    @Test
    fun getAssetNftFallsBackToApi() = runTest {
        val assetId = AssetId(Chain.Ethereum, "asset-1")
        every { nftDao.getAsset("ethereum_asset-1") } returns flowOf(null)
        coEvery { gemDeviceApiClient.getNFT("ethereum_asset-1") } returns nftAssetData("ethereum_asset-1", "collection-2")

        val result = subject.getAssetNft(assetId).first()

        assertEquals("collection-2", result.collection.id)
        assertEquals("ethereum_asset-1", result.assets.single().id)
        verify { nftDao.getAsset("ethereum_asset-1") }
        coVerify { gemDeviceApiClient.getNFT("ethereum_asset-1") }
    }

    @Test
    fun getAssetNftFallsBackToApiWhenCollectionIsMissing() = runTest {
        val assetId = AssetId(Chain.Ethereum, "asset-1")
        every { nftDao.getAsset("ethereum_asset-1") } returns flowOf(dbAsset("ethereum_asset-1", "collection-2"))
        every { nftDao.getCollection("collection-2") } returns flowOf(null)
        coEvery { gemDeviceApiClient.getNFT("ethereum_asset-1") } returns nftAssetData("ethereum_asset-1", "collection-3")

        val result = subject.getAssetNft(assetId).first()

        assertEquals("collection-3", result.collection.id)
        assertEquals("ethereum_asset-1", result.assets.single().id)
        verify { nftDao.getCollection("collection-2") }
        coVerify { gemDeviceApiClient.getNFT("ethereum_asset-1") }
    }
}

private fun dbCollection(id: String) = DbNFTCollection(
    id = id,
    name = id,
    chain = Chain.Ethereum,
    contractAddress = "0xcollection",
    imageUrl = "https://example.com/$id.png",
    previewImageUrl = "https://example.com/$id.png",
    originalSourceUrl = "https://example.com/$id.png",
    status = VerificationStatus.Verified,
)

private fun dbAsset(id: String, collectionId: String) = DbNFTAsset(
    id = id,
    collectionId = collectionId,
    tokenId = "1",
    tokenType = NFTType.ERC721,
    name = id,
    chain = Chain.Ethereum,
    contractAddress = "0xasset",
    imageUrl = "https://example.com/$id.png",
    previewImageUrl = "https://example.com/$id.png",
    originalSourceUrl = "https://example.com/$id.png",
)

private fun nftAssetData(assetId: String, collectionId: String) = NFTAssetData(
    collection = NFTCollection(
        id = collectionId,
        name = collectionId,
        description = null,
        chain = Chain.Ethereum,
        contractAddress = "0xcollection",
        images = NFTImages(NFTResource("https://example.com/$collectionId.png", "")),
        status = VerificationStatus.Verified,
        links = emptyList(),
    ),
    asset = NFTAsset(
        id = assetId,
        collectionId = collectionId,
        contractAddress = "0xasset",
        tokenId = "1",
        tokenType = NFTType.ERC721,
        name = assetId,
        description = null,
        chain = Chain.Ethereum,
        resource = NFTResource("", ""),
        images = NFTImages(NFTResource("https://example.com/$assetId.png", "")),
        attributes = emptyList(),
    ),
)

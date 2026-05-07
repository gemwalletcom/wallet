package com.gemwallet.android.data.repositories.nft

import com.gemwallet.android.data.service.store.database.NftDao
import com.gemwallet.android.data.service.store.database.entities.DbNFTAsset
import com.gemwallet.android.data.service.store.database.entities.DbNFTCollection
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NFTType
import com.wallet.core.primitives.VerificationStatus
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
    fun getAssetNftReadsRequestedWallet() = runTest {
        val assetId = AssetId(Chain.Ethereum, "asset-1")
        every { nftDao.getAsset("wallet-2", "ethereum_asset-1") } returns flowOf(dbAsset("ethereum_asset-1", "collection-2"))
        every { nftDao.getCollection("wallet-2", "collection-2") } returns flowOf(dbCollection("collection-2"))

        val result = subject.getAssetNft(mockWalletId("wallet-2"), assetId).first()

        assertEquals("collection-2", result.collection.id)
        assertEquals("ethereum_asset-1", result.assets.single().id)
        verify { nftDao.getAsset("wallet-2", "ethereum_asset-1") }
        verify { nftDao.getCollection("wallet-2", "collection-2") }
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

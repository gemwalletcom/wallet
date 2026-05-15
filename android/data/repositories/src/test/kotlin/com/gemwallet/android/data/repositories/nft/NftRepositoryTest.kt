package com.gemwallet.android.data.repositories.nft

import com.gemwallet.android.data.service.store.database.NftDao
import com.gemwallet.android.data.service.store.database.entities.DbNFTAsset
import com.gemwallet.android.data.service.store.database.entities.DbNFTCollection
import com.gemwallet.android.data.services.gemapi.GemDeviceApiClient
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockNftAsset
import com.gemwallet.android.testkit.mockNftAssetData
import com.gemwallet.android.testkit.mockNftAssetId
import com.gemwallet.android.testkit.mockNftCollection
import com.gemwallet.android.testkit.mockNftCollectionId
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTCollectionId
import com.wallet.core.primitives.NFTType
import com.wallet.core.primitives.VerificationStatus
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test

class NftRepositoryTest {

    private val gemDeviceApiClient = mockk<GemDeviceApiClient>()
    private val nftDao = mockk<NftDao>()
    private val subject = NftRepository(gemDeviceApiClient, nftDao)

    private val collectionId = mockNftCollectionId()
    private val otherCollectionId = mockNftCollectionId(contractAddress = "0xother")
    private val assetId = mockNftAssetId()

    @Test
    fun getListNftReadsRequestedWallet() = runTest {
        every { nftDao.getCollections("wallet-1") } returns flowOf(listOf(dbCollection(collectionId)))
        every { nftDao.getAssets("wallet-1") } returns flowOf(listOf(dbAsset(assetId, collectionId)))

        val result = subject.getListNft(mockWalletId("wallet-1")).first()

        assertEquals(listOf(collectionId), result.map { it.collection.id })
        assertEquals(listOf(assetId), result.flatMap { it.assets }.map { it.id })
    }

    @Test
    fun getAssetNftReadsFromCache() = runTest {
        every { nftDao.getAsset(assetId) } returns flowOf(dbAsset(assetId, collectionId))
        every { nftDao.getCollection(collectionId) } returns flowOf(dbCollection(collectionId))

        val result = subject.getAssetNft(assetId).first()

        assertEquals(collectionId, result.collection.id)
        assertEquals(assetId, result.assets.single().id)
        coVerify(exactly = 0) { gemDeviceApiClient.getNFT(any()) }
    }

    @Test
    fun getAssetNftFallsBackToApiAndAddsToStore() = runTest {
        every { nftDao.getAsset(assetId) } returns flowOf(null)
        coEvery { gemDeviceApiClient.getNFT(assetId.toIdentifier()) } returns mockNftAssetData(
            collection = mockNftCollection(id = collectionId),
            asset = mockNftAsset(id = assetId, collectionId = collectionId),
        )
        coEvery { nftDao.add(any(), any()) } returns Unit

        val result = subject.getAssetNft(assetId).first()

        assertEquals(collectionId, result.collection.id)
        assertEquals(assetId, result.assets.single().id)
        coVerify { gemDeviceApiClient.getNFT(assetId.toIdentifier()) }
        coVerify {
            nftDao.add(
                collection = match { it.id == collectionId },
                asset = match { it.id == assetId },
            )
        }
    }

    @Test
    fun getAssetNftFallsBackToApiWhenCollectionIsMissing() = runTest {
        every { nftDao.getAsset(assetId) } returns flowOf(dbAsset(assetId, collectionId))
        every { nftDao.getCollection(collectionId) } returns flowOf(null)
        coEvery { gemDeviceApiClient.getNFT(assetId.toIdentifier()) } returns mockNftAssetData(
            collection = mockNftCollection(id = otherCollectionId),
            asset = mockNftAsset(id = assetId, collectionId = otherCollectionId),
        )
        coEvery { nftDao.add(any(), any()) } returns Unit

        val result = subject.getAssetNft(assetId).first()

        assertEquals(otherCollectionId, result.collection.id)
        assertEquals(assetId, result.assets.single().id)
        coVerify { gemDeviceApiClient.getNFT(assetId.toIdentifier()) }
    }
}

private fun dbCollection(id: NFTCollectionId) = DbNFTCollection(
    id = id,
    name = id.toIdentifier(),
    chain = id.chain,
    contractAddress = id.contractAddress,
    imageUrl = "",
    previewImageUrl = "",
    originalSourceUrl = "",
    status = VerificationStatus.Verified,
)

private fun dbAsset(id: NFTAssetId, collectionId: NFTCollectionId) = DbNFTAsset(
    id = id,
    collectionId = collectionId,
    tokenId = id.tokenId,
    tokenType = NFTType.ERC721,
    name = id.toIdentifier(),
    chain = id.chain,
    contractAddress = id.contractAddress,
    imageUrl = "",
    previewImageUrl = "",
    originalSourceUrl = "",
)

package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.data.services.gemstone.stores.GemstoneNftStore
import com.gemwallet.android.data.service.store.database.NftDao
import com.gemwallet.android.data.service.store.database.entities.DbNFTAsset
import com.gemwallet.android.data.service.store.database.entities.DbNFTCollection
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
import uniffi.gemstone.GemNftService

class NftCasesTest {

    private val nftService = mockk<GemNftService>()
    private val nftDao = mockk<NftDao>()
    private val nftStore = GemstoneNftStore(nftDao)
    private val getListNft = GetListNftImpl(nftStore)
    private val getAssetNft = GetAssetNftImpl(nftService, nftStore)

    private val collectionId = mockNftCollectionId()
    private val otherCollectionId = mockNftCollectionId(contractAddress = "0xother")
    private val assetId = mockNftAssetId()

    @Test
    fun getListNftReadsRequestedWallet() = runTest {
        every { nftDao.getCollections("wallet-1") } returns flowOf(listOf(dbCollection(collectionId)))
        every { nftDao.getAssets("wallet-1") } returns flowOf(listOf(dbAsset(assetId, collectionId)))

        val result = getListNft.getListNft(mockWalletId("wallet-1")).first()

        assertEquals(listOf(collectionId), result.map { it.collection.id })
        assertEquals(listOf(assetId), result.flatMap { it.assets }.map { it.id })
    }

    @Test
    fun getAssetNftReadsFromCache() = runTest {
        every { nftDao.getAsset(assetId) } returns flowOf(dbAsset(assetId, collectionId))
        every { nftDao.getCollection(collectionId) } returns flowOf(dbCollection(collectionId))

        val result = getAssetNft.getAssetNft(assetId).first()

        assertEquals(collectionId, result.collection.id)
        assertEquals(assetId, result.assets.single().id)
        coVerify(exactly = 0) { nftService.ensureAsset(any()) }
    }

    @Test
    fun getAssetNftFallsBackToService() = runTest {
        every { nftDao.getAsset(assetId) } returns flowOf(null)
        coEvery { nftService.ensureAsset(assetId.toIdentifier()) } returns mockNftAssetData(
            collection = mockNftCollection(id = collectionId),
            asset = mockNftAsset(id = assetId, collectionId = collectionId),
        ).toGem()

        val result = getAssetNft.getAssetNft(assetId).first()

        assertEquals(collectionId, result.collection.id)
        assertEquals(assetId, result.assets.single().id)
        coVerify { nftService.ensureAsset(assetId.toIdentifier()) }
    }

    @Test
    fun getAssetNftFallsBackToServiceWhenCollectionIsMissing() = runTest {
        every { nftDao.getAsset(assetId) } returns flowOf(dbAsset(assetId, collectionId))
        every { nftDao.getCollection(collectionId) } returns flowOf(null)
        coEvery { nftService.ensureAsset(assetId.toIdentifier()) } returns mockNftAssetData(
            collection = mockNftCollection(id = otherCollectionId),
            asset = mockNftAsset(id = assetId, collectionId = otherCollectionId),
        ).toGem()

        val result = getAssetNft.getAssetNft(assetId).first()

        assertEquals(otherCollectionId, result.collection.id)
        assertEquals(assetId, result.assets.single().id)
        coVerify { nftService.ensureAsset(assetId.toIdentifier()) }
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

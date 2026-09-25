package com.gemwallet.android.data.coordinators.nft

import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.NFTAssetQuery
import com.gemwallet.android.domains.nft.NFTAssetDetails
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockNftAsset
import com.gemwallet.android.testkit.mockNftAssetData
import com.gemwallet.android.testkit.mockNftAssetId
import com.gemwallet.android.testkit.mockNftCollection
import com.gemwallet.android.testkit.mockNftCollectionId
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemCollectibleServiceInterface
import uniffi.gemstone.GemNftServiceInterface

class GetNftAssetDetailsImplTest {

    private val assetId = mockNftAssetId()
    private val stored = mockNftAssetData(
        collection = mockNftCollection(id = mockNftCollectionId()),
        asset = mockNftAsset(id = assetId, collectionId = mockNftCollectionId()),
    )
    private val ensured = mockNftAssetData(
        collection = mockNftCollection(id = mockNftCollectionId(contractAddress = "0xother")),
        asset = mockNftAsset(id = assetId, collectionId = mockNftCollectionId(contractAddress = "0xother")),
    )
    private val wallet = mockWallet(id = "wallet-1")
    private val getSession = mockk<GetSession> { every { this@mockk.invoke() } returns MutableStateFlow(mockSession(wallet = wallet)) }
    private val nftService = mockk<GemNftServiceInterface>()
    private val collectibleService = mockk<GemCollectibleServiceInterface>(relaxed = true)
    private val nftAssetQuery = mockk<NFTAssetQuery>()

    private fun subject() = GetNftAssetDetailsImpl(getSession, nftAssetQuery, nftService, collectibleService)

    @Test
    fun aStoredAssetIsDetailedWithItsOwnershipInTheSessionWallet() = runTest {
        every { nftAssetQuery("wallet-1", assetId) } returns flowOf(NFTAssetDetails(assetData = stored, isOwned = true))

        val result = subject()(assetId, canSaveImage = true).first()

        assertEquals(stored.collection, result.collection)
        assertEquals(stored.asset, result.asset)
        verify { collectibleService.details(wallet.type.toGem(), stored.toGem(), true, true) }
        coVerify(exactly = 0) { nftService.ensureAsset(any()) }
    }

    @Test
    fun aMissingAssetIsEnsuredThroughTheServiceAndNotOwned() = runTest {
        every { nftAssetQuery("wallet-1", assetId) } returns flowOf(null)
        coEvery { nftService.ensureAsset(assetId.toIdentifier()) } returns ensured.toGem()

        val result = subject()(assetId, canSaveImage = false).first()

        assertEquals(ensured.collection, result.collection)
        assertEquals(ensured.asset, result.asset)
        verify { collectibleService.details(wallet.type.toGem(), ensured.toGem(), false, false) }
    }
}

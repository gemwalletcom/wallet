package com.gemwallet.android.data.repositories.tokens

import com.gemwallet.android.data.repositories.prices.PricesRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.DbAssetBasicUpdate
import com.gemwallet.android.data.service.store.database.entities.DbPrice
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetBasic
import com.gemwallet.android.testkit.mockAssetEthereum
import com.gemwallet.android.testkit.mockSession
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.slot
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemAssetsService
import uniffi.gemstone.GemSearchService

class TokensRepositoryTest {

    private val wallet = mockWallet(id = "wallet-1")
    private val assetsDao = mockk<AssetsDao>(relaxed = true)
    private val pricesDao = mockk<PricesDao>(relaxed = true)
    private val pricesRepository = mockk<PricesRepository>(relaxed = true)
    private val sessionRepository = mockk<SessionRepository> {
        every { session() } returns MutableStateFlow(mockSession(wallet = wallet))
    }
    private val searchService = mockk<GemSearchService>()
    private val assetsService = mockk<GemAssetsService>()

    private val subject = TokensRepository(
        assetsDao = assetsDao,
        pricesDao = pricesDao,
        pricesRepository = pricesRepository,
        sessionRepository = sessionRepository,
        searchService = searchService,
        assetsService = assetsService,
    )

    @Test
    fun search_usesCoreSearchAssetsForTheSessionWallet() = runTest {
        coEvery { searchService.searchAssets(wallet.toJson(), "btc", Currency.USD.toJson()) } returns listOf(mockAssetBasic().toJson())
        coEvery { searchService.searchAssets(wallet.toJson(), "none", Currency.USD.toJson()) } returns emptyList()

        assertTrue(subject.search(query = "btc", currency = Currency.USD))
        assertFalse(subject.search(query = "none", currency = Currency.USD))
        assertFalse(subject.search(query = "", currency = Currency.USD))
    }

    @Test
    fun searchByAssetIds_storesCoreAssets() = runTest {
        val asset = mockAsset()
        val assetBasic = mockAssetBasic(asset = asset, rank = 100)
        val updates = slot<List<DbAssetBasicUpdate>>()
        coEvery { assetsService.getAssets(listOf(asset.id.toIdentifier()), null) } returns listOf(assetBasic.toJson())

        val result = subject.search(
            assetIds = listOf(asset.id),
            currency = Currency.USD,
        )

        assertTrue(result)
        coVerify { assetsDao.updateBasicAssets(capture(updates)) }
        assertEquals(100, updates.captured.single().rank)
    }

    @Test
    fun syncAssetPrices_fetchesOnlyAssetsMissingFromCache() = runTest {
        val cached = mockAsset()
        val missing = mockAssetEthereum()
        val missingBasic = mockAssetBasic(asset = missing)
        coEvery {
            pricesDao.getByAssets(listOf(cached.id.toIdentifier(), missing.id.toIdentifier()))
        } returns listOf(DbPrice(assetId = cached.id.toIdentifier(), currency = Currency.USD))
        coEvery { assetsService.getAssets(listOf(missing.id.toIdentifier()), null) } returns listOf(missingBasic.toJson())

        subject(listOf(cached.id, missing.id), Currency.USD)

        coVerify(exactly = 1) { assetsService.getAssets(listOf(missing.id.toIdentifier()), null) }
    }

    @Test
    fun syncAssetPrices_skipsApiCallWhenAllCached() = runTest {
        val asset = mockAsset()
        coEvery {
            pricesDao.getByAssets(listOf(asset.id.toIdentifier()))
        } returns listOf(DbPrice(assetId = asset.id.toIdentifier(), currency = Currency.USD))

        subject(listOf(asset.id), Currency.USD)

        coVerify(exactly = 0) { assetsService.getAssets(any(), any()) }
    }

    @Test
    fun syncAssetPrices_emptyList_isNoOp() = runTest {
        subject(emptyList(), Currency.USD)

        coVerify(exactly = 0) { pricesDao.getByAssets(any()) }
        coVerify(exactly = 0) { assetsService.getAssets(any(), any()) }
    }
}

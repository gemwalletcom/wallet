package com.gemwallet.android.data.repositories.tokens

import com.gemwallet.android.application.assets.coordinators.GemSearch
import com.gemwallet.android.blockchain.services.TokenService
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.service.store.database.AssetListDao
import com.gemwallet.android.data.service.store.database.SearchDao
import com.gemwallet.android.domains.search.WalletSearchTag
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetBasic
import com.gemwallet.android.testkit.mockPerpetual
import com.gemwallet.android.testkit.mockSearchResponse
import com.wallet.core.primitives.AssetList
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.PerpetualSearchData
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.mockk
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertTrue
import org.junit.Test

class WalletSearchTokensTest {

    private val tokensRepository = mockk<TokensRepository>(relaxed = true)
    private val gemSearch = mockk<GemSearch>()
    private val perpetualRepository = mockk<PerpetualRepository>(relaxed = true)
    private val searchDao = mockk<SearchDao>(relaxed = true)
    private val assetListDao = mockk<AssetListDao>(relaxed = true)
    private val tokenService = mockk<TokenService>(relaxed = true)

    private val subject = WalletSearchTokens(
        tokensRepository = tokensRepository,
        gemSearch = gemSearch,
        perpetualRepository = perpetualRepository,
        searchDao = searchDao,
        assetListDao = assetListDao,
        tokenService = tokenService,
    )

    @Test
    fun search_ingestsPerpetualsAndStoresPerpPriority() = runTest {
        coEvery {
            gemSearch.search(query = "btc", chains = emptyList(), scope = WalletSearchTag.All)
        } returns mockSearchResponse(
            assets = listOf(mockAssetBasic()),
            perpetuals = listOf(PerpetualSearchData(perpetual = mockPerpetual(), asset = mockAsset())),
        )

        val result = subject.search(
            query = "btc",
            currency = Currency.USD,
            chains = emptyList(),
            tags = emptyList(),
        )

        assertTrue(result)
        coVerify { perpetualRepository.putPerpetuals(any()) }
        coVerify { searchDao.put(match { records -> records.any { it.perpetualId != null } }) }
    }

    @Test
    fun search_storesListsWhenAll() = runTest {
        coEvery {
            gemSearch.search(query = "stocks", chains = emptyList(), scope = WalletSearchTag.All)
        } returns mockSearchResponse(
            assets = listOf(mockAssetBasic()),
            lists = listOf(AssetList(id = "stocks", name = "Stocks", count = 0u)),
        )

        subject.search(query = "stocks", currency = Currency.USD, chains = emptyList(), tags = emptyList())

        coVerify { assetListDao.upsert(match { records -> records.any { it.id == "stocks" } }) }
        coVerify { searchDao.put(match { records -> records.any { it.listId == "stocks" } }) }
    }

    @Test
    fun search_combinesOnChainNodeAssets() = runTest {
        coEvery {
            gemSearch.search(query = "0xabc", chains = emptyList(), scope = WalletSearchTag.All)
        } returns mockSearchResponse(assets = emptyList())
        coEvery { tokenService.search("0xabc", any()) } returns listOf(mockAssetBasic())

        val result = subject.search(query = "0xabc", currency = Currency.USD, chains = emptyList(), tags = emptyList())

        assertTrue(result)
        coVerify { searchDao.put(match { records -> records.any { it.assetId != null } }) }
    }

    @Test
    fun searchList_storesAssetsUnderListKey() = runTest {
        coEvery {
            gemSearch.search(query = "", chains = emptyList(), scope = WalletSearchTag.List("stocks"))
        } returns mockSearchResponse(assets = listOf(mockAssetBasic()))

        val result = subject.search(query = "", currency = Currency.USD, chains = emptyList(), scope = WalletSearchTag.List("stocks"))

        assertTrue(result)
        coVerify { searchDao.put(match { records -> records.all { it.assetId != null } }) }
    }

    @Test
    fun searchList_ingestsPerpetuals() = runTest {
        coEvery {
            gemSearch.search(query = "", chains = emptyList(), scope = WalletSearchTag.List("stocks"))
        } returns mockSearchResponse(
            assets = listOf(mockAssetBasic()),
            perpetuals = listOf(PerpetualSearchData(perpetual = mockPerpetual(), asset = mockAsset())),
        )

        val result = subject.search(query = "", currency = Currency.USD, chains = emptyList(), scope = WalletSearchTag.List("stocks"))

        assertTrue(result)
        coVerify { perpetualRepository.putPerpetuals(any()) }
        coVerify { searchDao.put(match { records -> records.any { it.perpetualId != null } }) }
    }

    @Test
    fun searchList_clearsStalePerpetualsWhenEmpty() = runTest {
        coEvery {
            gemSearch.search(query = "", chains = emptyList(), scope = WalletSearchTag.List("stocks"))
        } returns mockSearchResponse(assets = listOf(mockAssetBasic()))

        subject.search(query = "", currency = Currency.USD, chains = emptyList(), scope = WalletSearchTag.List("stocks"))

        coVerify { searchDao.deletePerpetuals(any()) }
    }

    @Test
    fun search_rethrowsCancellationWithoutStoring() = runTest {
        coEvery { gemSearch.search(any(), any(), any()) } throws CancellationException("cancelled")

        val result = runCatching {
            subject.search(query = "btc", currency = Currency.USD, chains = emptyList(), tags = emptyList())
        }

        assertTrue(result.exceptionOrNull() is CancellationException)
        coVerify(exactly = 0) { tokensRepository.updateAssets(any(), any()) }
        coVerify(exactly = 0) { searchDao.put(any()) }
        coVerify(exactly = 0) { searchDao.deleteAssets(any()) }
    }
}

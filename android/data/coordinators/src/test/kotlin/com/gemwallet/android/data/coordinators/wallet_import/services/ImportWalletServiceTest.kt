package com.gemwallet.android.data.coordinators.wallet_import.services

import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.application.wallet_import.coordinators.GetAvailableAssetIds
import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletConfiguration
import com.gemwallet.android.cases.device.SyncSubscription
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetInfo
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.coVerifyOrder
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceUntilIdle
import kotlinx.coroutines.test.runTest
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class ImportWalletServiceTest {

    private val sessionRepository = mockk<SessionRepository>()
    private val searchTokensCase = mockk<SearchTokensCase>(relaxed = true)
    private val assetsRepository = mockk<AssetsRepository>(relaxed = true)
    private val syncSubscription = mockk<SyncSubscription>(relaxed = true)
    private val syncTransactions = mockk<SyncTransactions>(relaxed = true)
    private val syncNfts = mockk<SyncNfts>(relaxed = true)
    private val walletConfigurationSync = mockk<SyncWalletConfiguration>(relaxed = true)

    @Test
    fun sync_linksImportedAssetsWithoutOverwritingAssetMetadata() = runTest {
        val token = mockAsset(chain = Chain.SmartChain, tokenId = "token", type = AssetType.BEP20)
        val tokenId = token.id.toIdentifier()
        val wallet = mockWallet(id = "wallet-1", accounts = listOf(mockAccount(chain = Chain.SmartChain)))
        val subject = service(availableAssetIds = listOf(tokenId))

        coEvery { sessionRepository.getCurrentCurrency() } returns Currency.USD
        every { assetsRepository.getTokensInfo(listOf(tokenId)) } returns flowOf(listOf(mockAssetInfo(asset = token)))

        subject.sync(wallet)
        advanceUntilIdle()

        coVerify { searchTokensCase.search(listOf(token.id), Currency.USD) }
        coVerify { assetsRepository.linkAssetToWallet("wallet-1", token.id, true) }
        coVerify { assetsRepository.updateBalances(token.id) }
        coVerify(exactly = 0) { assetsRepository.sync() }
    }

    @Test
    fun sync_syncsWalletConfigurationAfterSubscriptions() = runTest {
        val wallet = mockWallet(id = "wallet-1")
        val subject = service()

        coEvery { sessionRepository.getCurrentCurrency() } returns Currency.USD
        every { assetsRepository.getTokensInfo(emptyList()) } returns flowOf(emptyList())

        subject.sync(wallet)
        advanceUntilIdle()

        coVerifyOrder {
            syncSubscription.syncSubscription(listOf(wallet))
            walletConfigurationSync.sync(wallet.id)
        }
    }

    private fun TestScope.service(availableAssetIds: List<String> = emptyList()) = ImportWalletService(
        sessionRepository = sessionRepository,
        getAvailableAssetIds = GetAvailableAssetIds { availableAssetIds },
        searchTokensCase = searchTokensCase,
        assetsRepository = assetsRepository,
        syncSubscription = syncSubscription,
        syncTransactions = syncTransactions,
        syncNfts = syncNfts,
        walletConfigurationSync = walletConfigurationSync,
        scope = this,
    )
}

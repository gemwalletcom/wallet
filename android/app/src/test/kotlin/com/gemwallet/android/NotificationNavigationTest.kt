package com.gemwallet.android

import com.gemwallet.android.application.transactions.cases.CreateTransaction
import com.gemwallet.android.application.wallet.cases.GetWallet
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAccount
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletId
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualPositionRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.gemwallet.android.ui.navigation.routes.SupportRoute
import com.gemwallet.android.ui.navigation.routes.TransactionDetailsRoute
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemNavigationServiceInterface
import uniffi.gemstone.GemNavigationTarget
import uniffi.gemstone.GemPushNotification
import uniffi.gemstone.GemPushNotificationService

class NotificationNavigationTest {

    private val getWallet = mockk<GetWallet>(relaxed = true)
    private val createTransaction = mockk<CreateTransaction>(relaxed = true)

    private fun navigation(target: GemNavigationTarget): NotificationNavigation {
        val navigationService = mockk<GemNavigationServiceInterface> {
            coEvery { openNotification(any()) } returns target
        }
        return NotificationNavigation(getWallet, createTransaction, navigationService, GemPushNotificationService())
    }

    @Test
    fun `an asset Core opened becomes its route`() = runBlocking {
        val asset = mockAsset(chain = Chain.Ethereum)

        val routes = navigation(GemNavigationTarget.Asset(asset.toGem(), walletId = null, isPerpetual = false)).prepareNavigation(GemPushNotification.Rewards)

        assertEquals(listOf(AssetRoute(asset.id)), routes)
    }

    @Test
    fun `a perpetual opens its market before its position`() = runBlocking {
        val asset = mockAsset(chain = Chain.HyperCore, tokenId = "perpetual::UNI", type = AssetType.PERPETUAL)

        val routes = navigation(GemNavigationTarget.Asset(asset.toGem(), walletId = null, isPerpetual = true)).prepareNavigation(GemPushNotification.Rewards)

        assertEquals(listOf(PerpetualRoute, PerpetualPositionRoute(asset.id)), routes)
    }

    @Test
    fun `a transaction is stored before its route is offered`() = runBlocking {
        val assetId = mockAssetId(Chain.Ethereum)
        val walletId = mockWalletId()
        val asset = mockAsset(chain = assetId.chain, tokenId = assetId.tokenId)
        val transaction = mockTransaction(assetId = assetId)
        val wallet = mockWallet(id = walletId.id, accounts = listOf(mockAccount(chain = assetId.chain)))
        every { getWallet(wallet.id) } returns flowOf(wallet)
        coEvery { createTransaction.createNotificationTransaction(wallet, assetId, transaction) } returns asset

        val routes = navigation(GemNavigationTarget.Transaction(asset.toGem(), walletId.id, transaction.toGem(), isPerpetual = false))
            .prepareNavigation(
                GemPushNotification.Transaction(walletId = walletId.id, assetId = assetId.toIdentifier(), transaction = transaction.toGem()),
            )

        coVerify { createTransaction.createNotificationTransaction(wallet, assetId, transaction) }
        assertEquals(listOf(AssetRoute(asset.id), TransactionDetailsRoute(transaction.id)), routes)
    }

    @Test
    fun `support and rewards need no asset at all`() = runBlocking {
        assertEquals(listOf(SupportRoute), navigation(GemNavigationTarget.Support).prepareNavigation(GemPushNotification.Support))
        assertEquals(listOf(ReferralRoute(code = null)), navigation(GemNavigationTarget.Rewards(null)).prepareNavigation(GemPushNotification.Rewards))
    }

    @Test
    fun `a target Core could not prepare navigates nowhere`() = runBlocking {
        assertEquals(emptyList<Any>(), navigation(GemNavigationTarget.None).prepareNavigation(GemPushNotification.Test))
    }
}

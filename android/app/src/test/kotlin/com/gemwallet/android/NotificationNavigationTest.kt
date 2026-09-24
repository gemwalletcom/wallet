package com.gemwallet.android

import com.gemwallet.android.application.wallet.cases.GetWallet
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
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
import uniffi.gemstone.GemNavigationTab
import uniffi.gemstone.GemNavigationTarget
import uniffi.gemstone.GemPushNotification
import uniffi.gemstone.GemPushNotificationService

class NotificationNavigationTest {

    private val setCurrentWallet = mockk<SetCurrentWallet>(relaxed = true)

    private fun navigation(target: GemNavigationTarget): NotificationNavigation {
        val navigationService = mockk<GemNavigationServiceInterface> {
            coEvery { openNotification(any()) } returns target
        }
        return NotificationNavigation(navigationService, GemPushNotificationService(), setCurrentWallet)
    }

    @Test
    fun `an asset Core opened becomes its route`() = runBlocking {
        val asset = mockAsset(chain = Chain.Ethereum)

        val routes = navigation(GemNavigationTarget.Asset(asset.toGem(), walletId = null, isPerpetual = false)).prepareNavigation(GemPushNotification.Rewards).routes

        assertEquals(listOf(AssetRoute(asset.id)), routes)
    }

    @Test
    fun `a perpetual opens its market before its position`() = runBlocking {
        val asset = mockAsset(chain = Chain.HyperCore, tokenId = "perpetual::UNI", type = AssetType.PERPETUAL)

        val routes = navigation(GemNavigationTarget.Asset(asset.toGem(), walletId = null, isPerpetual = true)).prepareNavigation(GemPushNotification.Rewards).routes

        assertEquals(listOf(PerpetualRoute, PerpetualPositionRoute(asset.id)), routes)
    }

    @Test
    fun `a transaction Core opened routes to its details`() = runBlocking {
        val assetId = mockAssetId(Chain.Ethereum)
        val walletId = mockWalletId("multicoin_0x1")
        val asset = mockAsset(chain = assetId.chain, tokenId = assetId.tokenId)
        val transaction = mockTransaction(assetId = assetId)

        val routes = navigation(GemNavigationTarget.Transaction(asset.toGem(), walletId.id, transaction.toGem(), isPerpetual = false))
            .prepareNavigation(
                GemPushNotification.Transaction(walletId = walletId.id, assetId = assetId.toIdentifier(), transaction = transaction.toGem()),
            ).routes

        assertEquals(listOf(AssetRoute(asset.id), TransactionDetailsRoute(transaction.id)), routes)
        coVerify { setCurrentWallet.setCurrentWallet(walletId) }
    }

    @Test
    fun `support and rewards need no asset at all`() = runBlocking {
        assertEquals(PendingNavigation.Routes(listOf(SupportRoute), GemNavigationTab.SETTINGS), navigation(GemNavigationTarget.Support).prepareNavigation(GemPushNotification.Support))
        assertEquals(PendingNavigation.Routes(listOf(ReferralRoute(code = null)), GemNavigationTab.SETTINGS), navigation(GemNavigationTarget.Rewards(null)).prepareNavigation(GemPushNotification.Rewards))
        coVerify(exactly = 0) { setCurrentWallet.setCurrentWallet(any()) }
    }

    @Test
    fun `a target Core could not prepare navigates nowhere`() = runBlocking {
        assertEquals(emptyList<Any>(), navigation(GemNavigationTarget.None).prepareNavigation(GemPushNotification.Test).routes)
    }
}

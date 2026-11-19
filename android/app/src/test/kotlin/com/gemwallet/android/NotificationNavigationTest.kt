package com.gemwallet.android

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockTransaction
import com.gemwallet.android.testkit.mockWalletId
import com.gemwallet.android.ui.navigation.routes.AssetRoute
import com.gemwallet.android.ui.navigation.routes.ChartRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualsRoute
import com.gemwallet.android.ui.navigation.routes.RewardsRoute
import com.gemwallet.android.ui.navigation.routes.SupportRoute
import com.gemwallet.android.ui.navigation.routes.TransactionRoute
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import io.mockk.coEvery
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.runBlocking
import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.gemstone.GemAssetDestination
import uniffi.gemstone.GemNavigationServiceInterface
import uniffi.gemstone.GemNavigationTab
import uniffi.gemstone.GemNavigationTarget
import uniffi.gemstone.GemWalletSessionServiceInterface

class NotificationNavigationTest {

    private val walletSessionService = mockk<GemWalletSessionServiceInterface>(relaxed = true)
    private val type = "pushType"
    private val data = "{\"push\":\"data\"}"

    private fun navigation(target: GemNavigationTarget): NotificationNavigation {
        val navigationService = mockk<GemNavigationServiceInterface> {
            coEvery { openNotification(type, data) } returns target
        }
        return NotificationNavigation(navigationService, walletSessionService)
    }

    @Test
    fun `an asset Core opened becomes its route`() = runBlocking {
        val asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum))

        val routes = navigation(GemNavigationTarget.Asset(GemAssetDestination.Asset(asset.toGem()), walletId = null)).prepareNavigation(type, data).routes

        assertEquals(listOf(AssetRoute(asset.id)), routes)
    }

    @Test
    fun `a chart Core opened becomes its route on the wallet tab`() = runBlocking {
        val asset = mockAsset(id = mockAssetId(chain = Chain.Bitcoin))

        val destination = navigation(GemNavigationTarget.Chart(asset.toGem())).prepareNavigation(type, data)

        assertEquals(PendingNavigation.Routes(listOf(ChartRoute(asset.id)), GemNavigationTab.WALLET), destination)
    }

    @Test
    fun `a perpetual opens its market before its position`() = runBlocking {
        val asset = mockAsset(id = mockAssetId(chain = Chain.HyperCore, tokenId = "perpetual::UNI"), type = AssetType.PERPETUAL)

        val routes = navigation(GemNavigationTarget.Asset(GemAssetDestination.Perpetual(asset.toGem()), walletId = null)).prepareNavigation(type, data).routes

        assertEquals(listOf(PerpetualsRoute, PerpetualRoute(asset.id)), routes)
    }

    @Test
    fun `a transaction Core opened routes to its details`() = runBlocking {
        val assetId = mockAssetId(Chain.Ethereum)
        val walletId = mockWalletId(address = "0x1")
        val asset = mockAsset(id = assetId)
        val transaction = mockTransaction(assetId = assetId)

        val routes = navigation(GemNavigationTarget.Transaction(GemAssetDestination.Asset(asset.toGem()), walletId.id, transaction.id.toIdentifier()))
            .prepareNavigation(type, data).routes

        assertEquals(listOf(AssetRoute(asset.id), TransactionRoute(walletId, transaction.id)), routes)
        verify { walletSessionService.setCurrentWalletId(walletId.id) }
    }

    @Test
    fun `support and rewards need no asset at all`() = runBlocking {
        assertEquals(PendingNavigation.Routes(listOf(SupportRoute), GemNavigationTab.SETTINGS), navigation(GemNavigationTarget.Support).prepareNavigation(type, data))
        assertEquals(PendingNavigation.Routes(listOf(RewardsRoute(code = null)), GemNavigationTab.SETTINGS), navigation(GemNavigationTarget.Rewards(null)).prepareNavigation(type, data))
        verify(exactly = 0) { walletSessionService.setCurrentWalletId(any()) }
    }

    @Test
    fun `a target Core could not prepare navigates nowhere`() = runBlocking {
        assertEquals(emptyList<Any>(), navigation(GemNavigationTarget.None).prepareNavigation(type, data).routes)
    }
}

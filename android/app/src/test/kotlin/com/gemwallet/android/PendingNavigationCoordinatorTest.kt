package com.gemwallet.android

import android.content.Intent
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.PushNotificationField
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.ui.navigation.routes.FiatInputRoute
import com.gemwallet.android.ui.navigation.routes.PerpetualRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.FiatQuoteType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.yield
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.gemstone.GemDeeplinkService
import uniffi.gemstone.GemNavigationServiceInterface
import uniffi.gemstone.GemNavigationTab
import uniffi.gemstone.GemNavigationTarget

class PendingNavigationCoordinatorTest {

    private val notificationNavigation = mockk<NotificationNavigation>(relaxed = true)
    private val paymentNavigation = mockk<PaymentNavigation>(relaxed = true)
    private val navigationService = mockk<GemNavigationServiceInterface>(relaxed = true)
    private val coordinator = PendingNavigationCoordinator(notificationNavigation, paymentNavigation, navigationService, GemDeeplinkService())

    @Test
    fun buildRoutes_withoutPendingInput_isNoOp() = runTest {
        coordinator.buildRoutes(NoOpWalletConnect)

        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun buildRoutes_walletConnectPairing_invokesPairingHandlerAndClears() = runTest {
        val handler = RecordingWalletConnect()
        val uri = "wc:abc@2?relay-protocol=irn"
        coordinator.pendScan(uri)

        coordinator.buildRoutes(handler)

        assertEquals(listOf("pairing:$uri"), handler.events)
        assertNull("input must be cleared after handing off to wallet connect", coordinator.pendingNavigation.value)
    }

    @Test
    fun buildRoutes_walletConnectRequest_invokesRequestHandlerAndClears() = runTest {
        val handler = RecordingWalletConnect()
        val uri = "gem://wc?requestId=42"
        coordinator.pendScan(uri)

        coordinator.buildRoutes(handler)

        assertEquals(listOf("request"), handler.events)
        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun buildRoutes_webDeepLink_storesRoute() = runTest {
        val uri = "https://gemwallet.com/join/gemcoder"
        coEvery { navigationService.openDeeplink(any()) } returns GemNavigationTarget.Rewards("gemcoder")
        coordinator.pendScan(uri)

        coordinator.buildRoutes(NoOpWalletConnect)

        val routes = (coordinator.pendingNavigation.value as PendingNavigation.Routes).routes
        assertEquals(listOf(ReferralRoute(code = "gemcoder")), routes)
    }

    @Test
    fun buildRoutes_buyDeepLink_storesRouteWhenCoreOpensTheAsset() = runTest {
        val asset = mockAsset(id = mockAssetId(chain = Chain.Bitcoin))
        coEvery { navigationService.openDeeplink(any()) } returns GemNavigationTarget.Fiat(asset.toGem(), 100, FiatQuoteType.Buy.toGem())
        coordinator.pendScan("gem://tokens/bitcoin/buy?amount=100")

        coordinator.buildRoutes(NoOpWalletConnect)

        val routes = (coordinator.pendingNavigation.value as PendingNavigation.Routes).routes
        assertEquals(listOf(FiatInputRoute(asset.id, amount = 100, type = FiatQuoteType.Buy)), routes)
    }

    @Test
    fun buildRoutes_buyDeepLink_isDroppedWhenCoreRejectsTheAsset() = runTest {
        coEvery { navigationService.openDeeplink(any()) } returns GemNavigationTarget.None
        coordinator.pendScan("gem://tokens/bitcoin/buy")

        coordinator.buildRoutes(NoOpWalletConnect)

        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun buildRoutes_perpetualDeepLinks_storeRoute() = runTest {
        val uris = listOf(
            "gem://perpetuals",
            "https://gemwallet.com/perpetuals",
            "https://gemwallet.com/perpetuals/",
            "https://gemwallet.com/es/perpetuals/",
        )
        coEvery { navigationService.openDeeplink(any()) } returns GemNavigationTarget.Perpetuals

        uris.forEach { uri ->
            coordinator.pendScan(uri)
            coordinator.buildRoutes(NoOpWalletConnect)

            val routes = (coordinator.pendingNavigation.value as PendingNavigation.Routes).routes
            assertEquals(uri, listOf(PerpetualRoute), routes)
        }
    }

    @Test
    fun buildRoutes_unknownScan_clears() = runTest {
        val uri = "https://example.com/unknown"
        coordinator.pendScan(uri)

        coordinator.buildRoutes(NoOpWalletConnect)

        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun buildRoutes_paymentLink_showsLoadingUntilNavigationIsPrepared() = runTest {
        val uri = "solana:https%3A%2F%2Fexample.com%2Fpay"
        val release = CompletableDeferred<Unit>()
        coEvery { paymentNavigation.routes(any()) } coAnswers {
            release.await()
            emptyList()
        }
        coordinator.pendScan(uri)

        val build = launch { coordinator.buildRoutes(NoOpWalletConnect) }
        yield()

        assertEquals(PendingNavigation.Loading(PendingNavigation.FromScan(uri)), coordinator.pendingNavigation.value)

        release.complete(Unit)
        build.join()
        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun buildRoutes_notificationPayload_storesRouteFromNotificationNavigation() = runTest {
        val intent = intent(uri = null, hasNotificationPayload = true)
        val expected = PendingNavigation.Routes(listOf(ReferralRoute(code = "from-notification")), GemNavigationTab.SETTINGS)
        coEvery { notificationNavigation.prepareNavigation(intent) } returns expected
        coordinator.setIntent(intent)

        coordinator.buildRoutes(NoOpWalletConnect)

        coVerify(exactly = 1) { notificationNavigation.prepareNavigation(intent) }
        assertEquals(expected, coordinator.pendingNavigation.value)
    }

    @Test
    fun buildRoutes_notificationPayloadWithNoRoute_clears() = runTest {
        val intent = intent(uri = null, hasNotificationPayload = true)
        coEvery { notificationNavigation.prepareNavigation(intent) } returns PendingNavigation.Routes(emptyList())
        coordinator.setIntent(intent)

        coordinator.buildRoutes(NoOpWalletConnect)

        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun pendIntent_malformedExtras_isIgnored() {
        val intent = mockk<Intent>(relaxed = true)
        every { intent.dataString } returns null
        every { intent.hasExtra(any()) } throws RuntimeException("Parcelable encountered ClassNotFoundException reading a Serializable object")

        coordinator.pendIntent(intent)

        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun clear_clearsPendingNavigation() {
        coordinator.pendScan("https://example.com")

        coordinator.clear()

        assertNull(coordinator.pendingNavigation.value)
    }

    private fun intent(uri: String?, hasNotificationPayload: Boolean = false): Intent {
        val intent = mockk<Intent>(relaxed = true)
        every { intent.dataString } returns uri
        every { intent.hasExtra(PushNotificationField.Type.key) } returns hasNotificationPayload
        every { intent.hasExtra(PushNotificationField.Data.key) } returns false
        return intent
    }

    private object NoOpWalletConnect : PendingNavigationCoordinator.WalletConnectHandler {
        override fun onPairing(uri: String) = Unit
        override fun onRequest() = Unit
    }

    private class RecordingWalletConnect : PendingNavigationCoordinator.WalletConnectHandler {
        val events = mutableListOf<String>()
        override fun onPairing(uri: String) {
            events += "pairing:$uri"
        }
        override fun onRequest() {
            events += "request"
        }
    }
}

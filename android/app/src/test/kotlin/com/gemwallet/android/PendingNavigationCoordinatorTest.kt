package com.gemwallet.android

import android.content.Intent
import com.gemwallet.android.model.PushNotificationField
import com.gemwallet.android.ui.navigation.routes.PerpetualRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.wallet.core.primitives.Payment
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

class PendingNavigationCoordinatorTest {

    private val notificationNavigation = mockk<NotificationNavigation>(relaxed = true)
    private val paymentNavigation = mockk<PaymentNavigation>(relaxed = true)
    private val coordinator = PendingNavigationCoordinator(notificationNavigation, paymentNavigation, GemDeeplinkService())

    @Test
    fun buildRoutes_withoutPendingInput_isNoOp() = runTest {
        coordinator.buildRoutes(NoOpWalletConnect)

        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun buildRoutes_walletConnectPairing_invokesPairingHandlerAndClears() = runTest {
        val handler = RecordingWalletConnect()
        val uri = "wc:abc@2?relay-protocol=irn"
        coordinator.handleScan(uri)

        coordinator.buildRoutes(handler)

        assertEquals(listOf("pairing:$uri"), handler.events)
        assertNull("input must be cleared after handing off to wallet connect", coordinator.pendingNavigation.value)
    }

    @Test
    fun buildRoutes_walletConnectRequest_invokesRequestHandlerAndClears() = runTest {
        val handler = RecordingWalletConnect()
        val uri = "gem://wc?requestId=42"
        coordinator.handleScan(uri)

        coordinator.buildRoutes(handler)

        assertEquals(listOf("request"), handler.events)
        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun buildRoutes_webDeepLink_storesRoute() = runTest {
        val uri = "https://gemwallet.com/join/gemcoder"
        coordinator.handleScan(uri)

        coordinator.buildRoutes(NoOpWalletConnect)

        val routes = (coordinator.pendingNavigation.value as PendingNavigation.Routes).routes
        assertEquals(listOf(ReferralRoute(code = "gemcoder")), routes)
    }

    @Test
    fun buildRoutes_perpetualDeepLinks_storeRoute() = runTest {
        val uris = listOf(
            "gem://perpetuals",
            "https://gemwallet.com/perpetuals",
            "https://gemwallet.com/perpetuals/",
            "https://gemwallet.com/es/perpetuals/",
        )

        uris.forEach { uri ->
            coordinator.handleScan(uri)
            coordinator.buildRoutes(NoOpWalletConnect)

            val routes = (coordinator.pendingNavigation.value as PendingNavigation.Routes).routes
            assertEquals(uri, listOf(PerpetualRoute), routes)
        }
    }

    @Test
    fun buildRoutes_unknownScan_clears() = runTest {
        val uri = "https://example.com/unknown"
        coordinator.handleScan(uri)

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
        coordinator.handleScan(uri)

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
        val expected = listOf(ReferralRoute(code = "from-notification"))
        coEvery { notificationNavigation.prepareNavigation(intent) } returns expected
        coordinator.setIntent(intent)

        coordinator.buildRoutes(NoOpWalletConnect)

        coVerify(exactly = 1) { notificationNavigation.prepareNavigation(intent) }
        val routes = (coordinator.pendingNavigation.value as PendingNavigation.Routes).routes
        assertEquals(expected, routes)
    }

    @Test
    fun buildRoutes_notificationPayloadWithNoRoute_clears() = runTest {
        val intent = intent(uri = null, hasNotificationPayload = true)
        coEvery { notificationNavigation.prepareNavigation(intent) } returns emptyList()
        coordinator.setIntent(intent)

        coordinator.buildRoutes(NoOpWalletConnect)

        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun clear_clearsPendingNavigation() {
        coordinator.handleScan("https://example.com")

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
        override fun onPairing(uri: String) { events += "pairing:$uri" }
        override fun onRequest() { events += "request" }
    }
}

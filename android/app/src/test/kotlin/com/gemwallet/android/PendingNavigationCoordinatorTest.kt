package com.gemwallet.android

import android.content.Intent
import com.gemwallet.android.model.PushNotificationField
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.wallet.core.primitives.Payment
import com.wallet.core.primitives.PaymentLink
import com.wallet.core.primitives.PaymentLinkSolanaPayInner
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.yield
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.Deeplink
import uniffi.gemstone.UrlAction
import uniffi.gemstone.WalletConnectLink
import uniffi.gemstone.urlAction

class PendingNavigationCoordinatorTest {

    private val notificationNavigation = mockk<NotificationNavigation>(relaxed = true)
    private val paymentNavigation = mockk<PaymentNavigation>(relaxed = true)
    private val coordinator = PendingNavigationCoordinator(notificationNavigation, paymentNavigation)

    @Before
    fun setUp() = mockkStatic("uniffi.gemstone.GemstoneKt")

    @After
    fun tearDown() = unmockkStatic("uniffi.gemstone.GemstoneKt")

    @Test
    fun buildRoutes_withoutPendingInput_isNoOp() = runTest {
        coordinator.buildRoutes(NoOpWalletConnect)

        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun buildRoutes_walletConnectPairing_invokesPairingHandlerAndClears() = runTest {
        val handler = RecordingWalletConnect()
        val uri = "wc:abc@2?relay-protocol=irn"
        every { urlAction(uri) } returns UrlAction.WalletConnect(WalletConnectLink.Connect(uri))
        coordinator.handleScan(uri)

        coordinator.buildRoutes(handler)

        assertEquals(listOf("pairing:$uri"), handler.events)
        assertNull("input must be cleared after handing off to wallet connect", coordinator.pendingNavigation.value)
    }

    @Test
    fun buildRoutes_walletConnectRequest_invokesRequestHandlerAndClears() = runTest {
        val handler = RecordingWalletConnect()
        val uri = "gem://wc?requestId=42"
        every { urlAction(uri) } returns UrlAction.WalletConnect(WalletConnectLink.Request)
        coordinator.handleScan(uri)

        coordinator.buildRoutes(handler)

        assertEquals(listOf("request"), handler.events)
        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun buildRoutes_webDeepLink_storesRoute() = runTest {
        val uri = "https://gemwallet.com/join/gemcoder"
        every { urlAction(uri) } returns UrlAction.Deeplink(Deeplink.Rewards(code = "gemcoder"))
        coordinator.handleScan(uri)

        coordinator.buildRoutes(NoOpWalletConnect)

        val routes = (coordinator.pendingNavigation.value as PendingNavigation.Routes).routes
        assertEquals(listOf(ReferralRoute(code = "gemcoder")), routes)
    }

    @Test
    fun buildRoutes_unknownScan_clears() = runTest {
        val uri = "https://example.com/unknown"
        every { urlAction(uri) } returns null
        coordinator.handleScan(uri)

        coordinator.buildRoutes(NoOpWalletConnect)

        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun buildRoutes_paymentLink_showsLoadingUntilNavigationIsPrepared() = runTest {
        val uri = "solana:https%3A%2F%2Fexample.com%2Fpay"
        val payment: Payment = Payment.Link(PaymentLink.SolanaPay(PaymentLinkSolanaPayInner("https://example.com/pay")))
        val paymentJson = payment.toJson()
        val release = CompletableDeferred<Unit>()
        every { urlAction(uri) } returns UrlAction.Payment(paymentJson)
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

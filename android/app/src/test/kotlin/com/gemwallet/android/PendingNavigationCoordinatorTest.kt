package com.gemwallet.android

import android.content.Intent
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.model.PushNotificationField
import com.gemwallet.android.ui.navigation.routes.PaymentRoute
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import com.wallet.core.primitives.PaymentProviderName
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.mockkStatic
import io.mockk.unmockkStatic
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.gemstone.Deeplink
import uniffi.gemstone.GemPaymentLink
import uniffi.gemstone.GemPaymentProviderName
import uniffi.gemstone.UrlAction
import uniffi.gemstone.WalletConnectLink
import uniffi.gemstone.urlAction

class PendingNavigationCoordinatorTest {

    private val notificationNavigation = mockk<NotificationNavigation>(relaxed = true)
    private val userConfig = mockk<UserConfig>()
    private val coordinator = PendingNavigationCoordinator(notificationNavigation, userConfig)

    @Before
    fun setUp() = mockkStatic("uniffi.gemstone.GemstoneKt")

    @After
    fun tearDown() = unmockkStatic("uniffi.gemstone.GemstoneKt")

    @Test
    fun resolve_withoutPendingIntent_isNoOp() = runTest {
        coordinator.resolve(NoOpWalletConnect)

        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun resolve_walletConnectPairing_invokesPairingHandlerAndClears() = runTest {
        val handler = RecordingWalletConnect()
        val uri = "wc:abc@2?relay-protocol=irn"
        every { urlAction(uri) } returns UrlAction.WalletConnect(WalletConnectLink.Connect(uri))
        coordinator.setPendingIntentForTest(intent(uri = uri))

        coordinator.resolve(handler)

        assertEquals(listOf("pairing:$uri"), handler.events)
        assertNull("intent must be cleared after handing off to wallet connect", coordinator.pendingNavigation.value)
    }

    @Test
    fun resolve_walletConnectRequest_invokesRequestHandlerAndClears() = runTest {
        val handler = RecordingWalletConnect()
        val uri = "gem://wc?requestId=42"
        every { urlAction(uri) } returns UrlAction.WalletConnect(WalletConnectLink.Request)
        coordinator.setPendingIntentForTest(intent(uri = uri))

        coordinator.resolve(handler)

        assertEquals(listOf("request"), handler.events)
        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun resolve_webDeepLink_storesRoute() = runTest {
        val uri = "https://gemwallet.com/join/gemcoder"
        every { urlAction(uri) } returns UrlAction.Deeplink(Deeplink.Rewards(code = "gemcoder"))
        coordinator.setPendingIntentForTest(intent(uri = uri))

        coordinator.resolve(NoOpWalletConnect)

        val routes = (coordinator.pendingNavigation.value as PendingNavigation.Route).routes
        assertEquals(listOf(ReferralRoute(code = "gemcoder")), routes)
    }

    @Test
    fun resolve_paymentLink_storesRouteOnlyInDeveloperMode() = runTest {
        val uri = "https://pay.walletconnect.com/pay_1"
        every { urlAction(uri) } returns UrlAction.Payment(GemPaymentLink(GemPaymentProviderName.WALLET_CONNECT_PAY, "pay_1"))
        every { userConfig.developEnabled() } returns true
        coordinator.setPendingIntentForTest(intent(uri = uri))

        coordinator.resolve(NoOpWalletConnect)

        val routes = (coordinator.pendingNavigation.value as PendingNavigation.Route).routes
        assertEquals(listOf(PaymentRoute(PaymentProviderName.WalletConnectPay, "pay_1")), routes)

        every { userConfig.developEnabled() } returns false
        coordinator.setPendingIntentForTest(intent(uri = uri))

        coordinator.resolve(NoOpWalletConnect)

        assertNull("payment links must not navigate outside developer mode", coordinator.pendingNavigation.value)
    }

    @Test
    fun resolve_unknownIntentWithoutNotificationPayload_clears() = runTest {
        val uri = "https://example.com/unknown"
        every { urlAction(uri) } returns null
        coordinator.setPendingIntentForTest(intent(uri = uri))

        coordinator.resolve(NoOpWalletConnect)

        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun resolve_notificationPayload_storesRouteFromNotificationNavigation() = runTest {
        val intent = intent(uri = null, hasNotificationPayload = true)
        val expected = listOf(ReferralRoute(code = "from-notification"))
        coEvery { notificationNavigation.prepareNavigation(intent) } returns expected
        coordinator.setPendingIntentForTest(intent)

        coordinator.resolve(NoOpWalletConnect)

        coVerify(exactly = 1) { notificationNavigation.prepareNavigation(intent) }
        val routes = (coordinator.pendingNavigation.value as PendingNavigation.Route).routes
        assertEquals(expected, routes)
    }

    @Test
    fun resolve_notificationPayloadWithNoRoute_clears() = runTest {
        val intent = intent(uri = null, hasNotificationPayload = true)
        coEvery { notificationNavigation.prepareNavigation(intent) } returns emptyList()
        coordinator.setPendingIntentForTest(intent)

        coordinator.resolve(NoOpWalletConnect)

        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun consume_clearsPendingNavigation() {
        coordinator.setPendingIntentForTest(intent(uri = "https://example.com"))

        coordinator.consume()

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

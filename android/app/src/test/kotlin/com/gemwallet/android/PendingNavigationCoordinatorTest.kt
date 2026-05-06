package com.gemwallet.android

import android.content.Intent
import com.gemwallet.android.model.PushNotificationField
import com.gemwallet.android.ui.navigation.routes.ReferralRoute
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.test.runTest
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class PendingNavigationCoordinatorTest {

    private val notificationNavigation = mockk<NotificationNavigation>(relaxed = true)
    private val coordinator = PendingNavigationCoordinator(notificationNavigation)

    @Test
    fun resolve_withoutPendingIntent_isNoOp() = runTest {
        coordinator.resolve(NoOpWalletConnect)

        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun resolve_walletConnectPairing_invokesPairingHandlerAndClears() = runTest {
        val handler = RecordingWalletConnect()
        coordinator.setPendingIntentForTest(intent(uri = "wc:abc@2?relay-protocol=irn"))

        coordinator.resolve(handler)

        assertEquals(listOf("pairing:wc:abc@2?relay-protocol=irn"), handler.events)
        assertNull("intent must be cleared after handing off to wallet connect", coordinator.pendingNavigation.value)
    }

    @Test
    fun resolve_walletConnectRequest_invokesRequestHandlerAndClears() = runTest {
        val handler = RecordingWalletConnect()
        coordinator.setPendingIntentForTest(intent(uri = "gem://wc?requestId=42"))

        coordinator.resolve(handler)

        assertEquals(listOf("request"), handler.events)
        assertNull(coordinator.pendingNavigation.value)
    }

    @Test
    fun resolve_webDeepLink_storesRoute() = runTest {
        coordinator.setPendingIntentForTest(intent(uri = "https://gemwallet.com/join/gemcoder"))

        coordinator.resolve(NoOpWalletConnect)

        val routes = (coordinator.pendingNavigation.value as PendingNavigation.Route).routes
        assertEquals(listOf(ReferralRoute(code = "gemcoder")), routes)
    }

    @Test
    fun resolve_unknownIntentWithoutNotificationPayload_clears() = runTest {
        coordinator.setPendingIntentForTest(intent(uri = "https://example.com/unknown"))

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

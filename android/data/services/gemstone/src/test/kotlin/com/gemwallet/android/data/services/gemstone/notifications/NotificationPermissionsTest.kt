package com.gemwallet.android.data.services.gemstone.notifications

import org.junit.Assert.assertEquals
import org.junit.Test

class NotificationPermissionsTest {

    @Test
    fun `a missing notification permission opens the app page where it can be granted`() {
        assertEquals(NotificationSettingsTarget.AppDetails, notificationSettingsTarget(permissionGranted = false))
    }

    @Test
    fun `a granted permission opens the notification switch`() {
        assertEquals(NotificationSettingsTarget.NotificationSettings, notificationSettingsTarget(permissionGranted = true))
    }
}

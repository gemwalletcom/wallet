package com.gemwallet.android.data.password

import android.app.KeyguardManager
import android.content.Context
import android.content.Intent
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Assume.assumeNotNull
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import java.security.GeneralSecurityException
import java.security.KeyStore

@RunWith(AndroidJUnit4::class)
class PasswordKeysetInstrumentedTest {
    private val context = ApplicationProvider.getApplicationContext<Context>()
    private val config = TinkStoreConfig("gem_test_passwords", "passwords", "keyset", "gem_test_password_keyset", "gem_test_password_master")
    private val device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
    private lateinit var pin: String

    @Before
    fun prepare() {
        val testPin = InstrumentationRegistry.getArguments().getString("testPin")
        assumeNotNull(testPin)
        pin = testPin!!
        cleanupStore()
    }

    @After
    fun cleanup() {
        cleanupStore()
    }

    @Test
    fun protectedReadsNeedARecentAuthenticationAndFlagsCannotBypassIt() {
        store().putString("wallet", "secret")

        authenticate()
        PasswordKeyset(context, config).setAuthenticationRequired(true)
        assertTrue(PasswordKeyset(context, config).authenticationRequired)
        assertEquals("secret", store().getString("wallet"))

        waitForAuthenticationToExpire()
        assertThrows(GeneralSecurityException::class.java) { store().getString("wallet") }

        val preferences = context.getSharedPreferences(config.keysetPreferencesFileName, Context.MODE_PRIVATE)
        val stored = preferences.getString("keyset_authenticated", null)
        assertTrue(preferences.edit().remove("keyset_authenticated").commit())
        assertFalse(PasswordKeyset(context, config).authenticationRequired)
        assertThrows(Exception::class.java) { store().getString("wallet") }
        assertTrue(preferences.edit().putString("keyset_authenticated", stored).commit())

        authenticate()
        val reopened = PasswordKeyset(context, config)
        assertEquals("secret", store().getString("wallet"))
        reopened.setAuthenticationRequired(false)
        assertFalse(reopened.authenticationRequired)
        waitForAuthenticationToExpire()
        assertEquals("secret", store().getString("wallet"))
    }

    @Test
    fun removingTheDeviceLockInvalidatesProtectionWithoutReplacingTheKeyset() {
        val keyset = PasswordKeyset(context, config)
        keyset.get().encrypt(byteArrayOf(1, 2, 3), byteArrayOf())
        authenticate()
        keyset.setAuthenticationRequired(true)
        val preferences = context.getSharedPreferences(config.keysetPreferencesFileName, Context.MODE_PRIVATE)
        val stored = preferences.getString("keyset_authenticated", null)
        try {
            check(device.executeShellCommand("locksettings clear --old $pin").contains("Lock credential cleared"))
            assertThrows(GeneralSecurityException::class.java) { PasswordKeyset(context, config).get() }
            assertEquals(stored, preferences.getString("keyset_authenticated", null))
            assertTrue(PasswordKeyset(context, config).authenticationRequired)
        } finally {
            device.executeShellCommand("locksettings set-pin $pin")
        }
    }

    private fun store() = TinkEncryptedKeyValueStore(context, config, PasswordKeyset(context, config)::get)

    private fun authenticate() {
        val intent = checkNotNull(context.getSystemService(KeyguardManager::class.java).createConfirmDeviceCredentialIntent("Test", null))
        context.startActivity(intent.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
        val entry = checkNotNull(device.wait(Until.findObject(By.clazz("android.widget.EditText")), 10000)) { "Credential prompt did not appear" }
        entry.text = pin
        device.pressEnter()
        assertTrue(device.wait(Until.gone(By.clazz("android.widget.EditText")), 10000))
    }

    private fun waitForAuthenticationToExpire() {
        Thread.sleep((PASSWORD_AUTHENTICATION_WINDOW_SECONDS + 1) * 1000L)
    }

    private fun cleanupStore() {
        context.deleteSharedPreferences(config.keysetPreferencesFileName)
        context.deleteSharedPreferences(config.preferencesFileName)
        KeyStore.getInstance("AndroidKeyStore").apply {
            load(null)
            listOf(config.masterKeyAlias, "${config.masterKeyAlias}_authenticated").forEach(::deleteEntry)
        }
    }
}

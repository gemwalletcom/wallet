package com.gemwallet.android.data.password

import android.app.KeyguardManager
import android.content.Context
import android.security.keystore.KeyInfo
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.After
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Assume.assumeNotNull
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import java.security.GeneralSecurityException
import java.security.KeyStore
import javax.crypto.SecretKey
import javax.crypto.SecretKeyFactory

@RunWith(AndroidJUnit4::class)
class AuthenticatedKeysetAeadTest {
    private val alias = "gem_test_authenticated_keyset"
    private val keyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }

    @Before
    fun prepare() {
        assumeNotNull(InstrumentationRegistry.getArguments().getString("testPin"))
        val context = ApplicationProvider.getApplicationContext<Context>()
        check(context.getSystemService(KeyguardManager::class.java).isDeviceSecure) { "This test requires a test device with a screen lock" }
        keyStore.deleteEntry(alias)
    }

    @After
    fun cleanup() {
        keyStore.deleteEntry(alias)
    }

    @Test
    fun keyCannotBeUsedWithoutARecentAuthentication() {
        val encryption = AuthenticatedKeysetAead(alias)
        encryption.createKey()
        val key = keyStore.getKey(alias, null) as SecretKey
        val info = SecretKeyFactory.getInstance(key.algorithm, "AndroidKeyStore").getKeySpec(key, KeyInfo::class.java) as KeyInfo
        assertTrue(info.isUserAuthenticationRequired)
        assertTrue(info.userAuthenticationValidityDurationSeconds == PASSWORD_AUTHENTICATION_WINDOW_SECONDS)

        assertThrows(GeneralSecurityException::class.java) { encryption.encrypt(byteArrayOf(1, 2, 3), byteArrayOf()) }
    }

    @Test
    fun missingKeyIsNotRecreatedWhenReading() {
        assertThrows(GeneralSecurityException::class.java) { AuthenticatedKeysetAead(alias).decrypt(ByteArray(28), byteArrayOf()) }
        assertTrue(!keyStore.containsAlias(alias))
    }
}

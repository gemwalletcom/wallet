package com.gemwallet.android.data.password

import android.content.SharedPreferences
import com.gemwallet.android.math.hex
import com.google.crypto.tink.Aead
import com.google.crypto.tink.KeysetHandle
import com.google.crypto.tink.RegistryConfiguration
import com.google.crypto.tink.TinkProtoKeysetFormat
import com.google.crypto.tink.aead.AeadConfig
import com.google.crypto.tink.aead.AesGcmKeyManager
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import org.junit.Assert.assertThrows
import org.junit.Before
import org.junit.Test
import java.security.GeneralSecurityException

class EncryptedKeysetTest {
    private val preferences = mockk<SharedPreferences>()
    private val template = AesGcmKeyManager.aes256GcmTemplate()

    @Before
    fun setUp() {
        AeadConfig.register()
        every { preferences.getString("keyset", null) } returns null
    }

    @Test
    fun wrappingFailureDoesNotWrite() {
        val master = mockk<Aead>()
        every { master.encrypt(any(), any()) } throws GeneralSecurityException("Keystore unavailable")

        assertThrows(GeneralSecurityException::class.java) { encryptedKeyset(preferences, "keyset", master, template) }
        verify(exactly = 0) { preferences.edit() }
    }

    @Test
    fun failedCommitDoesNotReturnKeyset() {
        val master = KeysetHandle.generateNew(template).getPrimitive(RegistryConfiguration.get(), Aead::class.java)
        val editor = mockk<SharedPreferences.Editor>()
        every { preferences.edit() } returns editor
        every { editor.putString("keyset", any()) } returns editor
        every { editor.commit() } returns false

        assertThrows(IllegalStateException::class.java) { encryptedKeyset(preferences, "keyset", master, template) }
    }

    @Test
    fun wrongMasterKeyDoesNotReplaceEncryptedKeyset() {
        val master = KeysetHandle.generateNew(template).getPrimitive(RegistryConfiguration.get(), Aead::class.java)
        val wrongMaster = KeysetHandle.generateNew(template).getPrimitive(RegistryConfiguration.get(), Aead::class.java)
        val stored = TinkProtoKeysetFormat.serializeEncryptedKeyset(KeysetHandle.generateNew(template), master, byteArrayOf())
        every { preferences.getString("keyset", null) } returns stored.hex

        assertThrows(GeneralSecurityException::class.java) { encryptedKeyset(preferences, "keyset", wrongMaster, template) }
        verify(exactly = 0) { preferences.edit() }
    }
}

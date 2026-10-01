@file:Suppress("DEPRECATION")

package com.gemwallet.android.data.password

import android.content.Context
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.math.fromHex
import com.google.crypto.tink.InsecureSecretKeyAccess
import com.google.crypto.tink.TinkProtoKeysetFormat
import com.google.crypto.tink.aead.AeadConfig
import com.google.crypto.tink.aead.AesGcmKeyManager
import com.google.crypto.tink.integration.android.AndroidKeysetManager
import com.google.crypto.tink.integration.android.AndroidKeystoreKmsClient
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith
import java.io.File
import java.security.GeneralSecurityException
import java.security.SecureRandom
import java.util.UUID
import java.util.concurrent.CountDownLatch
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit

@RunWith(AndroidJUnit4::class)
class SecureStorageFailureInstrumentedTest {
    private val context = ApplicationProvider.getApplicationContext<Context>()
    private val name = "storage_failure_${UUID.randomUUID()}"
    private val config = TinkStoreConfig(name, name, "keyset", "${name}_keys", name)

    @After
    fun cleanup() {
        listOf(name, "${name}_keys", "${name}_legacy").forEach(context::deleteSharedPreferences)
        java.security.KeyStore.getInstance("AndroidKeyStore").apply { load(null) }.deleteEntry(name)
    }

    @Test
    fun corruptFilesDoNotCreateReplacementPasswordsOrKeysets() {
        for (fileName in listOf(name, "${name}_legacy", config.keysetPreferencesFileName)) {
            val file = preferencesFile(fileName)
            val corrupt = "<map><string name=\"secret\">truncated"
            file.writeText(corrupt)

            assertThrows(Exception::class.java) { passwordStore().getOrCreatePassword("wallet") }
            assertEquals(corrupt, file.readText())
            assertTrue(context.getSharedPreferences(name, Context.MODE_PRIVATE).all.isEmpty())
            assertTrue(file.delete())
        }
    }

    @Test
    fun corruptBackupIsNotDiscardedForAnEmptyPrimary() {
        val file = preferencesFile(name)
        file.writeText("<map />")
        val backup = File("${file.path}.bak")
        val corrupt = "<map><string"
        backup.writeText(corrupt)

        assertThrows(Exception::class.java) { passwordStore().getOrCreatePassword("wallet") }
        assertEquals(corrupt, backup.readText())
        assertEquals("<map />", file.readText())
    }

    @Test
    fun unreadableValuesAreNotTreatedAsMissing() {
        val file = preferencesFile(name)
        file.writeText("<map />")
        assertTrue(file.setReadable(false, false))
        try {
            assertThrows(Exception::class.java) { passwordStore().getOrCreatePassword("wallet") }
        } finally {
            assertTrue(file.setReadable(true, true))
        }
        assertEquals("<map />", file.readText())
    }

    @Test
    fun legacyBackupOnlyIsRecovered() {
        val sourceName = "${name}_legacy"
        val legacy = EncryptedSharedPreferences.create(
            context,
            sourceName,
            MasterKey.Builder(context).setKeyScheme(MasterKey.KeyScheme.AES256_GCM).build(),
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
        )
        assertTrue(legacy.edit().putString("wallet", "existing-password").commit())
        val target = preferencesFile("${name}_legacy")
        val contents = target.readBytes()
        assertTrue(context.deleteSharedPreferences(sourceName))
        File("${target.path}.bak").writeBytes(contents)
        assertEquals("existing-password", LegacyEncryptedPreferences(context, "${name}_legacy").getString("wallet"))
        assertTrue(target.isFile)
    }

    @Test
    fun missingLegacyKeysetDoesNotHidePassword() {
        val preferences = context.getSharedPreferences("${name}_legacy", Context.MODE_PRIVATE)
        assertTrue(preferences.edit().putString("encrypted-name", "encrypted-password").commit())
        assertThrows(IllegalStateException::class.java) { passwordStore().getOrCreatePassword("wallet") }
        assertEquals(setOf("encrypted-name"), preferences.all.keys)
    }

    @Test
    fun masterKeyFailureDoesNotPersistAKeysetOrPassword() {
        val provider = TinkAeadProvider(context, config) { throw GeneralSecurityException("Keystore unavailable") }
        val store = TinkEncryptedKeyValueStore(context, config, provider::get)
        assertThrows(GeneralSecurityException::class.java) { store.putString("wallet", "password") }
        assertTrue(context.getSharedPreferences(config.keysetPreferencesFileName, Context.MODE_PRIVATE).all.isEmpty())
        assertTrue(context.getSharedPreferences(name, Context.MODE_PRIVATE).all.isEmpty())
    }

    @Test
    fun plaintextKeysetIsRewrappedWithoutChangingKeys() {
        AeadConfig.register()
        val handle = AndroidKeysetManager.Builder()
            .withSharedPref(context, config.keysetName, config.keysetPreferencesFileName)
            .withKeyTemplate(AesGcmKeyManager.aes256GcmTemplate())
            .doNotUseKeystore()
            .build().keysetHandle
        val preferences = context.getSharedPreferences(config.keysetPreferencesFileName, Context.MODE_PRIVATE)
        val plaintext = preferences.getString(config.keysetName, null)!!
        TinkAeadProvider(context, config).get()
        val encrypted = preferences.getString(config.keysetName, null)!!.fromHex()
        val masterAead = AndroidKeystoreKmsClient().getAead("android-keystore://${config.masterKeyAlias}")
        assertTrue(handle.equalsKeyset(TinkProtoKeysetFormat.parseEncryptedKeyset(encrypted, masterAead, byteArrayOf())))
        assertThrows(GeneralSecurityException::class.java) { TinkProtoKeysetFormat.parseKeyset(encrypted, InsecureSecretKeyAccess.get()) }
        assertFalse(plaintext == preferences.getString(config.keysetName, null))
    }

    @Test
    fun encryptedValuesSurviveReopening() {
        TinkEncryptedKeyValueStore.create(context, config).putString("wallet", "existing-password")
        assertEquals("existing-password", TinkEncryptedKeyValueStore.create(context, config).getString("wallet"))
    }

    @Test
    fun concurrentLegacyReadsAndWritesPreserveBothPasswords() {
        val fileName = "${name}_legacy"
        val writer = LegacyEncryptedPreferences(context, fileName)
        writer.putString("wallet", "existing-password")
        val executor = Executors.newFixedThreadPool(2)
        val start = CountDownLatch(1)
        try {
            val writing = executor.submit {
                start.await()
                repeat(50) { writer.putString("other-wallet", "other-password-$it") }
            }
            val reading = executor.submit {
                start.await()
                repeat(50) { assertEquals("existing-password", LegacyEncryptedPreferences(context, fileName).getString("wallet")) }
            }
            start.countDown()
            writing.get(30, TimeUnit.SECONDS)
            reading.get(30, TimeUnit.SECONDS)
            assertEquals("other-password-49", writer.getString("other-wallet"))
        } finally {
            executor.shutdownNow()
        }
    }

    private fun passwordStore() = TinkPasswordStore(
        TinkEncryptedKeyValueStore.create(context, config),
        LegacyEncryptedPreferences(context, "${name}_legacy"),
        SecureRandom(),
    )

    private fun preferencesFile(fileName: String) = File(context.applicationInfo.dataDir, "shared_prefs/$fileName.xml").also { it.parentFile!!.mkdirs() }
}

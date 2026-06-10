package com.gemwallet.android.blockchain.operators.gemstone

import android.content.Context
import android.util.Log
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.math.fromHex
import com.gemwallet.android.testkit.KEYSTORE_TEST_PASSWORD
import com.gemwallet.android.testkit.TEST_PHRASE
import com.gemwallet.android.testkit.includeGemstoneLibs
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.GemImportType
import uniffi.gemstone.GemKeystore
import java.io.File
import kotlin.system.measureNanoTime

@RunWith(AndroidJUnit4::class)
class GemKeystoreBenchmarkTest {

    private val context = ApplicationProvider.getApplicationContext<Context>()
    private lateinit var baseDir: File

    @Before
    fun setUp() {
        baseDir = File(context.cacheDir, "gemk-benchmark").apply { deleteRecursively(); mkdirs() }
    }

    @After
    fun tearDown() {
        baseDir.deleteRecursively()
    }

    @Test
    fun benchmarkEncryptAndDecryptWithDefaultKdf() {
        val password = KEYSTORE_TEST_PASSWORD.fromHex()
        val import = GemImportType.SinglePhrase(words = TEST_PHRASE.split(" "), chain = "ethereum")

        GemKeystore(baseDir.path).use { keystore ->
            var keystoreId = keystore.createStore(import, password).keystoreId
            val encryptMillis = (1..ITERATIONS).map {
                keystore.delete(keystoreId)
                measureNanoTime { keystoreId = keystore.createStore(import, password).keystoreId } / 1_000_000
            }

            keystore.exportRecoveryPhrase(keystoreId, password)
            val decryptMillis = (1..ITERATIONS).map {
                measureNanoTime {
                    assertEquals(TEST_PHRASE, keystore.exportRecoveryPhrase(keystoreId, password).joinToString(" "))
                } / 1_000_000
            }

            Log.i(TAG, "encrypt(createStore) ms: $encryptMillis, median ${encryptMillis.sorted()[encryptMillis.size / 2]}")
            Log.i(TAG, "decrypt(exportRecoveryPhrase) ms: $decryptMillis, median ${decryptMillis.sorted()[decryptMillis.size / 2]}")
        }
    }

    private companion object {
        init {
            includeGemstoneLibs()
        }

        const val TAG = "GemKeystoreBenchmark"
        const val ITERATIONS = 5
    }
}

package com.gemwallet.android.blockchain.operators.gemstone

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import com.gemwallet.android.math.fromHex
import com.gemwallet.android.testkit.KEYSTORE_TEST_PASSWORD
import com.gemwallet.android.testkit.TEST_PHRASE
import com.gemwallet.android.testkit.includeGemstoneLibs
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.gemstone.GemImportType
import uniffi.gemstone.GemKeystore
import java.io.File
import java.util.concurrent.CyclicBarrier

@RunWith(AndroidJUnit4::class)
class GemKeystoreConcurrencyTest {

    private val context = ApplicationProvider.getApplicationContext<Context>()
    private lateinit var baseDir: File

    @Before
    fun setUp() {
        baseDir = File(context.cacheDir, "gemk-concurrency").apply { deleteRecursively(); mkdirs() }
    }

    @After
    fun tearDown() {
        baseDir.deleteRecursively()
    }

    @Test
    fun concurrentCreateReadDelete_acrossInstancesAndThreads_isThreadSafe() = runBlocking {
        val password = KEYSTORE_TEST_PASSWORD.fromHex()
        val import = GemImportType.SinglePhrase(words = TEST_PHRASE.split(" "), chain = "ethereum")

        val startCreate = CyclicBarrier(THREADS)
        val created = (0 until THREADS).map {
            async(Dispatchers.IO) {
                startCreate.await()
                GemKeystore(baseDir.path).use { it.createStore(import, password) }
            }
        }.awaitAll()

        val keystoreId = created.first().keystoreId
        assertTrue("concurrent creates of the same wallet must return one deterministic id", created.all { it.keystoreId == keystoreId })

        val files = File(baseDir, "v4").listFiles { file -> file.extension == "gemk" }.orEmpty()
        assertEquals("the race must leave exactly one keystore file, not duplicates", 1, files.size)

        val startRead = CyclicBarrier(THREADS)
        val phrases = (0 until THREADS).map {
            async(Dispatchers.IO) {
                startRead.await()
                GemKeystore(baseDir.path).use { it.exportRecoveryPhrase(keystoreId, password).joinToString(" ") }
            }
        }.awaitAll()
        assertTrue("every concurrent read must return the stored phrase", phrases.all { it == TEST_PHRASE })

        val startDelete = CyclicBarrier(THREADS)
        (0 until THREADS).map {
            async(Dispatchers.IO) {
                startDelete.await()
                GemKeystore(baseDir.path).use { runCatching { it.delete(keystoreId) } }
            }
        }.awaitAll()
        assertFalse("keystore file must be removed after concurrent delete", File(baseDir, "v4/$keystoreId.gemk").exists())
    }

    companion object {
        init {
            includeGemstoneLibs()
        }

        private const val THREADS = 8
    }
}

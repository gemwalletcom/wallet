package com.gemwallet.android.services

import android.content.Context
import android.content.SharedPreferences
import coil3.ImageLoader
import coil3.disk.DiskCache
import com.gemwallet.android.data.password.SecureStorageReset
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletId
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.coVerifyOrder
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.runBlocking
import okhttp3.Cache
import okhttp3.OkHttpClient
import org.junit.Assert.assertThrows
import org.junit.Test
import uniffi.gemstone.GemWalletServiceInterface

class LocalDataResetTest {

    private val wallets = listOf(mockWallet(id = mockWalletId("0x1")).toGem(), mockWallet(id = mockWalletId("0x2")).toGem())
    private val walletStore = mockk<GemstoneWalletStore> { coEvery { getWallets() } returns wallets }
    private val walletService = mockk<GemWalletServiceInterface>(relaxed = true)
    private val database = mockk<GemDatabase>(relaxed = true)
    private val httpCache = mockk<Cache>(relaxed = true)
    private val imageCache = mockk<DiskCache>(relaxed = true)
    private val secureStorage = mockk<SecureStorageReset>(relaxed = true)
    private val preferencesEditor = mockk<SharedPreferences.Editor> {
        every { clear() } returns this
        every { commit() } returns true
    }
    private val context = mockk<Context> {
        every { getSharedPreferences(any(), Context.MODE_PRIVATE) } returns mockk { every { edit() } returns preferencesEditor }
    }
    private val reset = LocalDataReset(
        walletStore = walletStore,
        walletService = walletService,
        database = database,
        httpClient = OkHttpClient.Builder().cache(httpCache).build(),
        imageLoader = mockk<ImageLoader> { every { diskCache } returns imageCache },
        secureStorage = secureStorage,
        context = context,
    )

    @Test
    fun reset_clearsEveryStoreAndTheSecureStorageLast() = runBlocking {
        reset.reset()

        coVerifyOrder {
            walletService.deleteWallet(wallets[0].id)
            walletService.deleteWallet(wallets[1].id)
            database.clearAllTables()
            secureStorage.clear()
        }
        listOf("gemstone_preferences", "gateway_preferences", "device-info", "config").forEach { name ->
            verify { context.getSharedPreferences(name, Context.MODE_PRIVATE) }
        }
        verify(exactly = 4) { preferencesEditor.commit() }
        verify { httpCache.evictAll() }
        verify { imageCache.clear() }
    }

    @Test
    fun failedReset_keepsTheSecureStorage() {
        coEvery { walletService.deleteWallet(wallets[1].id) } throws IllegalStateException("Wallet store failed")

        assertThrows(IllegalStateException::class.java) { runBlocking { reset.reset() } }

        coVerify(exactly = 0) { secureStorage.clear() }
    }
}

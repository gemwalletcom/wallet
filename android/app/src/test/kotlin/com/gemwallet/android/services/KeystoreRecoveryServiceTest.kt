package com.gemwallet.android.services

import android.content.Context
import android.content.SharedPreferences
import coil3.ImageLoader
import coil3.disk.DiskCache
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.data.password.SecureStorageRecovery
import com.gemwallet.android.data.password.TinkPasswordStore
import com.gemwallet.android.data.services.store.database.GemDatabase
import com.gemwallet.android.data.services.store.queries.WalletsQuery
import com.gemwallet.android.testkit.mockWallet
import com.gemwallet.android.testkit.mockWalletId
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.coVerifyOrder
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import okhttp3.Cache
import okhttp3.OkHttpClient
import org.junit.Assert.assertFalse
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import uniffi.gemstone.GemWalletServiceInterface

class KeystoreRecoveryServiceTest {

    private val sharedPasswordWallet = mockWallet(id = mockWalletId("0x1"))
    private val v3Wallet = mockWallet(id = mockWalletId("0x2"))
    private val viewWallet = mockWallet(id = mockWalletId("0x3"), type = WalletType.View)
    private val walletsQuery = mockk<WalletsQuery>().also { every { it() } returns flowOf(listOf(sharedPasswordWallet, v3Wallet, viewWallet)) }
    private val walletService = mockk<GemWalletServiceInterface>(relaxed = true)
    private val passwords = mockk<TinkPasswordStore>()
    private val migrateV3KeystoreService = mockk<MigrateV3KeystoreService> {
        every { needsMigration(any()) } answers { firstArg<Wallet>() == v3Wallet }
    }
    private val database = mockk<GemDatabase>(relaxed = true)
    private val httpCache = mockk<Cache>(relaxed = true)
    private val imageCache = mockk<DiskCache>(relaxed = true)
    private val secureStorage = mockk<SecureStorageRecovery>(relaxed = true)
    private val preferencesEditor = mockk<SharedPreferences.Editor> {
        every { clear() } returns this
        every { commit() } returns true
    }
    private val context = mockk<Context> {
        every { getSharedPreferences(any(), Context.MODE_PRIVATE) } returns mockk { every { edit() } returns preferencesEditor }
    }
    private val recovery = KeystoreRecoveryService(
        walletsQuery = walletsQuery,
        walletService = walletService,
        passwords = passwords,
        migrateV3KeystoreService = migrateV3KeystoreService,
        database = database,
        httpClient = OkHttpClient.Builder().cache(httpCache).build(),
        imageLoader = mockk<ImageLoader> { every { diskCache } returns imageCache },
        secureStorage = secureStorage,
        context = context,
    )

    @Test
    fun readablePasswords_repairOnlyTheUnreadableLockStores() = runBlocking {
        every { passwords.canReadPasswords(listOf(PasswordStore.Keys.Password.key, v3Wallet.id.id)) } returns true

        assertTrue(recovery.repairLockStores())

        verify { secureStorage.clearUnreadableLockStores() }
        coVerify(exactly = 0) { secureStorage.clear() }
        coVerify(exactly = 0) { walletService.deleteWallet(any()) }
    }

    @Test
    fun unreadablePasswords_repairNothing() = runBlocking {
        every { passwords.canReadPasswords(any()) } returns false

        assertFalse(recovery.repairLockStores())

        verify(exactly = 0) { secureStorage.clearUnreadableLockStores() }
    }

    @Test
    fun reset_clearsEveryStoreAndTheSecureStorageLast() = runBlocking {
        recovery.resetLocalData()

        coVerifyOrder {
            walletService.deleteWallet(sharedPasswordWallet.id.id)
            walletService.deleteWallet(v3Wallet.id.id)
            walletService.deleteWallet(viewWallet.id.id)
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
        coEvery { walletService.deleteWallet(v3Wallet.id.id) } throws IllegalStateException("Wallet store failed")

        assertThrows(IllegalStateException::class.java) { runBlocking { recovery.resetLocalData() } }

        coVerify(exactly = 0) { secureStorage.clear() }
    }
}

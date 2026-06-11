package com.gemwallet.android.services

import android.content.Context
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.operators.MigrateKeystoreOperator
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.math.fromHex
import com.gemwallet.android.testkit.KEYSTORE_TEST_ETH_ADDRESS
import com.gemwallet.android.testkit.KEYSTORE_TEST_PASSWORD
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.io.File
import java.nio.file.Files

class MigrateV3KeystoreServiceTest {

    private lateinit var baseDir: File
    private lateinit var passwordStore: PasswordStore
    private lateinit var walletsRepository: WalletsRepository
    private lateinit var migrateKeystoreOperator: MigrateKeystoreOperator
    private lateinit var service: MigrateV3KeystoreService

    @Before
    fun setUp() {
        baseDir = Files.createTempDirectory("migrate-v3-keystore").toFile()
        val context = mockk<Context> {
            every { dataDir } returns baseDir
        }
        passwordStore = mockk()
        walletsRepository = mockk()
        migrateKeystoreOperator = mockk()
        service = MigrateV3KeystoreService(context, walletsRepository, passwordStore, migrateKeystoreOperator)
    }

    @After
    fun tearDown() {
        baseDir.deleteRecursively()
    }

    @Test
    fun migrateWallet_invokesOperatorWithDecodedPasswordAndZeroizesIt() = runBlocking {
        val walletId = WalletId("privateKey_ethereum_$KEYSTORE_TEST_ETH_ADDRESS")
        val current = mockWallet(id = walletId.id, type = WalletType.PrivateKey, source = WalletSource.Import)
        every { walletsRepository.getAll() } answers { flowOf(listOf(current)) }
        prepareV3File(walletId)
        var capturedLegacyPassword = byteArrayOf()
        var capturedNewPassword = byteArrayOf()
        every { migrateKeystoreOperator(any(), any(), any(), any()) } answers {
            capturedLegacyPassword = secondArg()
            capturedNewPassword = thirdArg()
            assertArrayEquals(KEYSTORE_TEST_PASSWORD.fromHex(), capturedLegacyPassword)
            assertArrayEquals(KEYSTORE_TEST_PASSWORD.fromHex(), capturedNewPassword)
            "keystore-id"
        }

        service()

        verify(exactly = 1) {
            migrateKeystoreOperator(File(baseDir, walletId.id).path, any(), any(), walletId.id)
        }
        assertTrue(capturedLegacyPassword.all { it == 0.toByte() })
        assertTrue(capturedNewPassword.all { it == 0.toByte() })
        coVerify(exactly = 0) { walletsRepository.updateWallet(any()) }
    }

    @Test
    fun migrateWithEmptyPassword_skipsMigration() = runBlocking {
        listOf("", "0x").forEach { password ->
            baseDir.deleteRecursively()
            baseDir.mkdirs()
            val walletId = WalletId("privateKey_ethereum_$KEYSTORE_TEST_ETH_ADDRESS")
            val current = mockWallet(id = walletId.id, type = WalletType.PrivateKey, source = WalletSource.Import)
            every { walletsRepository.getAll() } answers { flowOf(listOf(current)) }
            prepareV3File(walletId, password)

            service()

            assertTrue("v3 file must remain pending", File(baseDir, walletId.id).exists())
            verify(exactly = 0) { migrateKeystoreOperator(any(), any(), any(), any()) }
        }
    }

    private fun prepareV3File(walletId: WalletId, password: String = KEYSTORE_TEST_PASSWORD) {
        File(baseDir, walletId.id).writeText("{}")
        every { passwordStore.getPassword(walletId.id) } returns password
    }

}

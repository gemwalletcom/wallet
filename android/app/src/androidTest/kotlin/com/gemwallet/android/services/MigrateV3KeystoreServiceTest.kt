package com.gemwallet.android.services

import android.content.Context
import androidx.test.core.app.ApplicationProvider
import androidx.test.platform.app.InstrumentationRegistry
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.operators.gemstone.GemMigrateKeystoreOperator
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.ext.keystoreId
import com.gemwallet.android.ext.v4KeystorePasswordBytes
import com.gemwallet.android.math.hex
import com.gemwallet.android.testkit.KEYSTORE_TEST_ETH_ADDRESS
import com.gemwallet.android.testkit.KEYSTORE_TEST_PASSWORD
import com.gemwallet.android.testkit.includeGemstoneLibs
import com.gemwallet.android.testkit.mockWallet
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
import io.mockk.coVerify
import io.mockk.every
import io.mockk.mockk
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.runBlocking
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import java.io.File

class MigrateV3KeystoreServiceTest {

    private val context = ApplicationProvider.getApplicationContext<Context>()
    private val baseDir = context.dataDir
    private val passwordStore = mockk<PasswordStore>()
    private val walletsRepository = mockk<WalletsRepository>()
    private val service = MigrateV3KeystoreService(context, walletsRepository, passwordStore, GemMigrateKeystoreOperator(baseDir.toString()))

    private fun loadKey(wallet: Wallet, chain: Chain, password: String): ByteArray =
        uniffi.gemstone.GemKeystore(baseDir.toString()).use { it.privateKey(wallet.keystoreId, chain.string, password.v4KeystorePasswordBytes()) }

    @Before
    fun setUp() = cleanup()

    @After
    fun tearDown() = cleanup()

    @Test
    fun migrateMnemonicWallet_createsV4AtDeterministicIdAndIsIdempotent() = runBlocking {
        val walletId = WalletId("multicoin_$KEYSTORE_TEST_ETH_ADDRESS")
        val current = mockWallet(id = walletId.id, type = WalletType.Multicoin, source = WalletSource.Import)
        every { walletsRepository.getAll() } answers { flowOf(listOf(current)) }
        prepareV3File(walletId, "v3_android_mnemonic.json")

        service()

        val keystoreId = uniffi.gemstone.keystoreIdForWallet(walletId.id)
        assertTrue("v4 file must exist at the deterministic id", File(baseDir, "$keystoreId.json").exists())
        assertFalse("v3 file must be deleted after a verified migration", File(baseDir, walletId.id).exists())
        assertEquals(EXPECTED_PRIVATE_KEY, loadKey(current, Chain.Ethereum, KEYSTORE_TEST_PASSWORD).hex)

        service()
        assertEquals(EXPECTED_PRIVATE_KEY, loadKey(current, Chain.Ethereum, KEYSTORE_TEST_PASSWORD).hex)
        coVerify(exactly = 0) { walletsRepository.updateWallet(any()) }
    }

    @Test
    fun migratePrivateKeyWallet_createsV4AtDeterministicIdAndIsIdempotent() = runBlocking {
        val walletId = WalletId("privateKey_ethereum_$KEYSTORE_TEST_ETH_ADDRESS")
        val current = mockWallet(id = walletId.id, type = WalletType.PrivateKey, source = WalletSource.Import)
        every { walletsRepository.getAll() } answers { flowOf(listOf(current)) }
        prepareV3File(walletId, "v3_android_private_key.json")

        service()

        val keystoreId = uniffi.gemstone.keystoreIdForWallet(walletId.id)
        assertTrue("v4 file must exist at the deterministic id", File(baseDir, "$keystoreId.json").exists())
        assertFalse("v3 file must be deleted after a verified migration", File(baseDir, walletId.id).exists())
        assertEquals(EXPECTED_PRIVATE_KEY, loadKey(current, Chain.Ethereum, KEYSTORE_TEST_PASSWORD).hex)

        service()
        assertEquals(EXPECTED_PRIVATE_KEY, loadKey(current, Chain.Ethereum, KEYSTORE_TEST_PASSWORD).hex)
        coVerify(exactly = 0) { walletsRepository.updateWallet(any()) }
    }

    @Test
    fun migrateV3WithEmptyScryptSalt_createsV4AndRecoversKey() = runBlocking {
        val walletId = WalletId("multicoin_$KEYSTORE_TEST_ETH_ADDRESS")
        val current = mockWallet(id = walletId.id, type = WalletType.Multicoin, source = WalletSource.Import)
        every { walletsRepository.getAll() } answers { flowOf(listOf(current)) }
        prepareV3File(walletId, "v3_android_empty_salt_mnemonic.json")

        service()

        val keystoreId = uniffi.gemstone.keystoreIdForWallet(walletId.id)
        assertTrue("v4 file must exist after migrating an empty-salt v3 file", File(baseDir, "$keystoreId.json").exists())
        assertFalse("v3 file must be deleted after a verified migration", File(baseDir, walletId.id).exists())
        assertEquals(EXPECTED_PRIVATE_KEY, loadKey(current, Chain.Ethereum, KEYSTORE_TEST_PASSWORD).hex)
    }

    private fun prepareV3File(walletId: WalletId, assetName: String) {
        val fixture = InstrumentationRegistry.getInstrumentation().context.assets
            .open(assetName).bufferedReader().use { it.readText() }
        File(baseDir, walletId.id).writeText(fixture)
        every { passwordStore.getPassword(walletId.id) } returns KEYSTORE_TEST_PASSWORD
    }

    private fun cleanup() {
        baseDir.listFiles { file -> file.extension == "json" }?.forEach { it.delete() }
        File(baseDir, "multicoin_$KEYSTORE_TEST_ETH_ADDRESS").delete()
        File(baseDir, "privateKey_ethereum_$KEYSTORE_TEST_ETH_ADDRESS").delete()
    }

    companion object {
        init {
            includeGemstoneLibs()
        }

        private const val EXPECTED_PRIVATE_KEY = "ae8794f84919b14ff9d1f0f7cf490a4c04e608de16864f53fe8b40af127b9da3"
    }
}

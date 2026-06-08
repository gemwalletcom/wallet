package com.gemwallet.android.services

import android.content.Context
import android.util.Log
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.blockchain.operators.MigrateKeystoreOperator
import com.gemwallet.android.data.repositories.wallets.WalletsRepository
import com.gemwallet.android.ext.keystoreId
import com.gemwallet.android.math.fromHex
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.withContext
import java.io.File
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class MigrateV3KeystoreService @Inject constructor(
    @ApplicationContext private val context: Context,
    private val walletsRepository: WalletsRepository,
    private val passwordStore: PasswordStore,
    private val migrateKeystoreOperator: MigrateKeystoreOperator,
) {
    private val baseDir: String get() = context.dataDir.toString()

    suspend operator fun invoke() = withContext(Dispatchers.IO) {
        walletsRepository.getAll().firstOrNull().orEmpty().forEach { wallet ->
            if (needsMigration(wallet)) {
                runCatching { migrate(wallet) }
                    .onFailure { Log.e(TAG, "v3 keystore migration failed for ${wallet.id.id}", it) }
            }
        }
    }

    private fun migrate(wallet: Wallet) {
        val legacyFile = File(baseDir, wallet.id.id)
        if (!legacyFile.exists()) return

        val passwordBytes = passwordStore.getPassword(wallet.id.id).fromHex()
        try {
            // Writes v4 at the deterministic id. The v3 file is left in place, wallet deletion removes every copy.
            migrateKeystoreOperator(legacyFile.path, passwordBytes, passwordBytes, wallet.keystoreId)
        } finally {
            passwordBytes.fill(0)
        }
    }

    private fun needsMigration(wallet: Wallet): Boolean =
        wallet.type != WalletType.View &&
            File(baseDir, wallet.id.id).exists() &&
            !File(baseDir, "v4/${wallet.keystoreId}.gemk").exists()

    private companion object {
        const val TAG = "MigrateV3Keystore"
    }
}

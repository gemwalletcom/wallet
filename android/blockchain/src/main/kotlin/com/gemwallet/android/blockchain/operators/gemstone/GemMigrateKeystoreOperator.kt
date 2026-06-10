package com.gemwallet.android.blockchain.operators.gemstone

import com.gemwallet.android.blockchain.operators.MigrateKeystoreOperator
import uniffi.gemstone.GemKeystore

class GemMigrateKeystoreOperator(
    private val baseDir: String,
) : MigrateKeystoreOperator {

    override fun invoke(
        legacyPath: String,
        legacyPassword: ByteArray,
        newPassword: ByteArray,
        walletId: String,
    ): String = GemKeystore(baseDir).use { keystore ->
        keystore.migrateV3(legacyPath, legacyPassword, newPassword, walletId).keystoreId
    }
}

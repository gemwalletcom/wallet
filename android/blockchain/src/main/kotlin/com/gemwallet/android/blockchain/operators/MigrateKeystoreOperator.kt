package com.gemwallet.android.blockchain.operators

interface MigrateKeystoreOperator {
    operator fun invoke(
        legacyPath: String,
        legacyPassword: ByteArray,
        newPassword: ByteArray,
        keystoreId: String,
    ): String
}

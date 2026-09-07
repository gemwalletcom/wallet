package com.gemwallet.android.data.services.gemstone.keystore

interface MigrateKeystoreOperator {
    operator fun invoke(
        legacyPath: String,
        legacyPassword: ByteArray,
        newPassword: ByteArray,
        walletId: String,
    ): String
}

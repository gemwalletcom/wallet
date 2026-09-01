package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.application.PasswordStore
import uniffi.gemstone.GemKeystorePassword

class GemstoneKeystorePassword(
    private val passwordStore: PasswordStore,
) : GemKeystorePassword {

    override fun getPassword(createIfMissing: Boolean): String {
        val key = PasswordStore.Keys.Password.key
        return if (createIfMissing) passwordStore.getOrCreatePassword(key) else passwordStore.getPassword(key)
    }

    override fun getWalletPassword(walletId: String): String? =
        if (passwordStore.hasPassword(walletId)) passwordStore.getPassword(walletId) else null

    override fun deleteWalletPassword(walletId: String) {
        passwordStore.removePassword(walletId)
    }
}

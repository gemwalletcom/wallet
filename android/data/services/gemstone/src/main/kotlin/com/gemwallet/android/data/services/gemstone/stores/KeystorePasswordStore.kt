package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.application.PasswordNotFoundException
import com.gemwallet.android.application.PasswordStore
import uniffi.gemstone.GemKeystorePassword

class GemstoneKeystorePassword(
    private val passwordStore: PasswordStore,
) : GemKeystorePassword {

    override fun getPassword(walletId: String, createIfMissing: Boolean): String {
        val password = try {
            passwordStore.getPassword(walletId)
        } catch (_: PasswordNotFoundException) {
            val passwordKey = PasswordStore.Keys.Password.key
            if (createIfMissing) {
                passwordStore.getOrCreatePassword(passwordKey).also {
                    // Keep per-wallet entries authoritative. Current direct readers and Tink-based rollback
                    // builds still resolve passwords by wallet id, so new wallets need this compatibility alias.
                    passwordStore.putPassword(walletId, it)
                }
            } else {
                passwordStore.getPassword(passwordKey)
            }
        }
        return password
    }

    override fun deletePassword(walletId: String) {
        passwordStore.removePassword(walletId)
    }
}

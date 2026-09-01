package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.getKeystorePassword
import com.gemwallet.android.application.getOrCreateKeystorePassword
import uniffi.gemstone.GemKeystorePassword

class GemstoneKeystorePassword(
    private val passwordStore: PasswordStore,
) : GemKeystorePassword {

    override fun getPassword(createIfMissing: Boolean): String {
        return if (createIfMissing) passwordStore.getOrCreateKeystorePassword() else passwordStore.getKeystorePassword()
    }

    override fun getWalletPassword(walletId: String): String? =
        if (passwordStore.hasPassword(walletId)) passwordStore.getPassword(walletId) else null

    override fun deleteWalletPassword(walletId: String) {
        passwordStore.removePassword(walletId)
    }
}

package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.ext.v4KeystorePasswordBytes
import uniffi.gemstone.GemKeystorePassword

class GemstoneKeystorePassword(
    private val passwordStore: PasswordStore,
) : GemKeystorePassword {

    override fun getPassword(walletId: String, createIfMissing: Boolean): ByteArray {
        val password = if (createIfMissing) passwordStore.createPassword(walletId) else passwordStore.getPassword(walletId)
        return password.v4KeystorePasswordBytes()
    }
}

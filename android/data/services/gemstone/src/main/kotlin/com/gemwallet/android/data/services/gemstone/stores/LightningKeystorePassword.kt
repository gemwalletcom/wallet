package com.gemwallet.android.data.services.gemstone.stores

import android.security.keystore.UserNotAuthenticatedException
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.getKeystorePassword
import uniffi.gemstone.GemKeystoreAuthentication
import uniffi.gemstone.GemKeystorePassword
import uniffi.gemstone.GemServiceException

class LightningKeystorePassword(private val passwordStore: PasswordStore, private val deviceAuthentication: DeviceAuthentication) : GemKeystorePassword {

    override fun getPassword(createIfMissing: Boolean): String = try {
        passwordStore.getKeystorePassword()
    } catch (error: Exception) {
        if (error.isUserNotAuthenticated()) throw GemServiceException.Cancelled()
        throw error
    }

    override fun getWalletPassword(walletId: String): String? = null

    override fun deleteWalletPassword(walletId: String) = Unit

    override fun authentication(): GemKeystoreAuthentication = deviceAuthentication.current()

    private fun Throwable.isUserNotAuthenticated(): Boolean = generateSequence(this) { it.cause }.any { it is UserNotAuthenticatedException }
}

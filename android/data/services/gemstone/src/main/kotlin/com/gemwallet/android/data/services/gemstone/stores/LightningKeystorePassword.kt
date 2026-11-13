package com.gemwallet.android.data.services.gemstone.stores

import android.security.keystore.UserNotAuthenticatedException
import uniffi.gemstone.GemKeystorePassword
import uniffi.gemstone.GemServiceException

class LightningKeystorePassword(private val password: GemKeystorePassword) : GemKeystorePassword by password {

    override fun getPassword(createIfMissing: Boolean): String = try {
        password.getPassword(createIfMissing)
    } catch (error: Exception) {
        if (error.isUserNotAuthenticated()) throw GemServiceException.Cancelled()
        throw error
    }

    private fun Throwable.isUserNotAuthenticated(): Boolean = generateSequence(this) { it.cause }.any { it is UserNotAuthenticatedException }
}

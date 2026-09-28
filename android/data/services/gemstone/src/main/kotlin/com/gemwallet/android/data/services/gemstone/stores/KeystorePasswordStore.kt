package com.gemwallet.android.data.services.gemstone.stores

import android.content.Context
import android.os.Build
import androidx.biometric.BiometricManager
import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.getKeystorePassword
import com.gemwallet.android.application.getOrCreateKeystorePassword
import com.gemwallet.android.application.security.cases.SecurityPreferences
import dagger.hilt.android.qualifiers.ApplicationContext
import uniffi.gemstone.GemKeystoreAuthentication
import uniffi.gemstone.GemKeystorePassword
import javax.inject.Inject

class GemstoneKeystorePassword(private val passwordStore: PasswordStore, private val deviceAuthentication: DeviceAuthentication) : GemKeystorePassword {

    override fun getPassword(createIfMissing: Boolean): String = if (createIfMissing) passwordStore.getOrCreateKeystorePassword() else passwordStore.getKeystorePassword()

    override fun getWalletPassword(walletId: String): String? = if (passwordStore.hasPassword(walletId)) passwordStore.getPassword(walletId) else null

    override fun deleteWalletPassword(walletId: String) {
        passwordStore.removePassword(walletId)
    }

    override fun authentication(): GemKeystoreAuthentication = deviceAuthentication.current()
}

class DeviceAuthentication(private val securityPreferences: SecurityPreferences, private val hasBiometrics: () -> Boolean) {

    @Inject
    constructor(@ApplicationContext context: Context, securityPreferences: SecurityPreferences) : this(
        securityPreferences,
        { BiometricManager.from(context).canAuthenticate(biometricStrength) == BiometricManager.BIOMETRIC_SUCCESS },
    )

    fun current(): GemKeystoreAuthentication = when {
        !securityPreferences.authRequired() -> GemKeystoreAuthentication.NONE
        hasBiometrics() -> GemKeystoreAuthentication.BIOMETRICS
        else -> GemKeystoreAuthentication.PASSCODE
    }

    private companion object {
        val biometricStrength = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) BiometricManager.Authenticators.BIOMETRIC_STRONG else BiometricManager.Authenticators.BIOMETRIC_WEAK
    }
}

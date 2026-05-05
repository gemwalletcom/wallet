package com.gemwallet.android

import android.os.Build
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricManager.Authenticators.BIOMETRIC_STRONG
import androidx.biometric.BiometricManager.Authenticators.BIOMETRIC_WEAK
import androidx.biometric.BiometricManager.Authenticators.DEVICE_CREDENTIAL
import androidx.biometric.BiometricPrompt
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Duration.Companion.seconds

internal object SystemAuthPolicy {
    private val transientRetryDelay = 1.seconds
    private val lockoutRetryDelay = 30.seconds

    val authRequestTimeout = 5.minutes
    val authRequestRestartDelay = 500.milliseconds

    val allowedAuthenticators = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
        BIOMETRIC_STRONG or DEVICE_CREDENTIAL
    } else {
        BIOMETRIC_WEAK or DEVICE_CREDENTIAL
    }

    fun initialRetryDelay(errorCode: Int): Duration? = when (errorCode) {
        BiometricPrompt.ERROR_CANCELED,
        BiometricPrompt.ERROR_NEGATIVE_BUTTON,
        BiometricPrompt.ERROR_TIMEOUT,
        BiometricPrompt.ERROR_USER_CANCELED -> Duration.ZERO
        BiometricPrompt.ERROR_HW_UNAVAILABLE,
        BiometricPrompt.ERROR_UNABLE_TO_PROCESS,
        BiometricPrompt.ERROR_VENDOR -> transientRetryDelay
        BiometricPrompt.ERROR_LOCKOUT -> lockoutRetryDelay
        else -> null
    }

    fun isEnrollmentMissing(canAuthenticateResult: Int): Boolean {
        return canAuthenticateResult == BiometricManager.BIOMETRIC_ERROR_NONE_ENROLLED
    }
}

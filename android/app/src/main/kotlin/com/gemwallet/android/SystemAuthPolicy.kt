package com.gemwallet.android

import android.os.Build
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricManager.Authenticators.BIOMETRIC_STRONG
import androidx.biometric.BiometricManager.Authenticators.BIOMETRIC_WEAK
import androidx.biometric.BiometricManager.Authenticators.DEVICE_CREDENTIAL
import androidx.biometric.BiometricPrompt
import uniffi.gemstone.GemAuthPromptOutcome
import uniffi.gemstone.GemSecurityService
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.minutes

internal object SystemAuthPolicy {
    private val securityService = GemSecurityService()

    val authRequestTimeout = 5.minutes
    val authRequestRestartDelay = 500.milliseconds

    val allowedAuthenticators = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
        BIOMETRIC_STRONG or DEVICE_CREDENTIAL
    } else {
        BIOMETRIC_WEAK or DEVICE_CREDENTIAL
    }

    fun initialRetryDelay(errorCode: Int): Duration? =
        securityService.authRetryDelayMilliseconds(promptOutcome(errorCode))?.toLong()?.milliseconds

    private fun promptOutcome(errorCode: Int): GemAuthPromptOutcome = when (errorCode) {
        BiometricPrompt.ERROR_CANCELED -> GemAuthPromptOutcome.CANCELLED_BY_SYSTEM
        BiometricPrompt.ERROR_NEGATIVE_BUTTON,
        BiometricPrompt.ERROR_TIMEOUT,
        BiometricPrompt.ERROR_USER_CANCELED -> GemAuthPromptOutcome.CANCELLED_BY_USER
        BiometricPrompt.ERROR_HW_UNAVAILABLE,
        BiometricPrompt.ERROR_UNABLE_TO_PROCESS,
        BiometricPrompt.ERROR_VENDOR -> GemAuthPromptOutcome.TRANSIENT
        BiometricPrompt.ERROR_LOCKOUT -> GemAuthPromptOutcome.LOCKED_OUT
        BiometricPrompt.ERROR_NO_BIOMETRICS,
        BiometricPrompt.ERROR_HW_NOT_PRESENT -> GemAuthPromptOutcome.UNAVAILABLE
        else -> GemAuthPromptOutcome.FAILED
    }

    fun isEnrollmentMissing(canAuthenticateResult: Int): Boolean {
        return canAuthenticateResult == BiometricManager.BIOMETRIC_ERROR_NONE_ENROLLED
    }
}

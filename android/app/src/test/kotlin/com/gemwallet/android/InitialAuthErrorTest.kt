package com.gemwallet.android

import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds

class InitialAuthErrorTest {

    @Test
    fun userAndTimeoutErrors_retryImmediately() {
        listOf(
            BiometricPrompt.ERROR_CANCELED,
            BiometricPrompt.ERROR_NEGATIVE_BUTTON,
            BiometricPrompt.ERROR_TIMEOUT,
            BiometricPrompt.ERROR_USER_CANCELED,
        ).forEach { errorCode ->
            assertEquals(Duration.ZERO, SystemAuthPolicy.initialRetryDelay(errorCode))
        }
    }

    @Test
    fun recoverableSystemErrors_retryAfterDelay() {
        listOf(
            BiometricPrompt.ERROR_HW_UNAVAILABLE,
            BiometricPrompt.ERROR_UNABLE_TO_PROCESS,
            BiometricPrompt.ERROR_VENDOR,
        ).forEach { errorCode ->
            assertEquals(1.seconds, SystemAuthPolicy.initialRetryDelay(errorCode))
        }
        assertEquals(30.seconds, SystemAuthPolicy.initialRetryDelay(BiometricPrompt.ERROR_LOCKOUT))
    }

    @Test
    fun unrecoverableSystemErrors_doNotRetry() {
        listOf(
            BiometricPrompt.ERROR_HW_NOT_PRESENT,
            BiometricPrompt.ERROR_LOCKOUT_PERMANENT,
            BiometricPrompt.ERROR_NO_BIOMETRICS,
            BiometricPrompt.ERROR_NO_DEVICE_CREDENTIAL,
            BiometricPrompt.ERROR_NO_SPACE,
            BiometricPrompt.ERROR_SECURITY_UPDATE_REQUIRED,
        ).forEach { errorCode ->
            assertNull(SystemAuthPolicy.initialRetryDelay(errorCode))
        }
    }

    @Test
    fun systemAuthEnrollmentMissing_onlyForNoneEnrolled() {
        assertTrue(SystemAuthPolicy.isEnrollmentMissing(BiometricManager.BIOMETRIC_ERROR_NONE_ENROLLED))

        listOf(
            BiometricManager.BIOMETRIC_SUCCESS,
            BiometricManager.BIOMETRIC_ERROR_HW_UNAVAILABLE,
            BiometricManager.BIOMETRIC_ERROR_NO_HARDWARE,
        ).forEach { canAuthenticateResult ->
            assertFalse(SystemAuthPolicy.isEnrollmentMissing(canAuthenticateResult))
        }
    }
}

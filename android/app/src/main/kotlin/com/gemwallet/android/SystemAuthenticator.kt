package com.gemwallet.android

import android.app.admin.DevicePolicyManager
import android.content.Intent
import android.provider.Settings
import android.widget.Toast
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import androidx.lifecycle.LifecycleDestroyedException
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.withResumed
import com.gemwallet.android.features.settings.viewmodels.lock.LockViewModel
import com.gemwallet.android.features.settings.viewmodels.lock.models.AuthState
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.model.requiresConfirmation
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.text
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlin.time.Duration

internal class SystemAuthenticator(private val activity: FragmentActivity, private val lockViewModel: LockViewModel, private val privacyCover: PrivacyCover) {
    private val _enrollmentMissing = MutableStateFlow(false)
    private val authRequests = AuthRequestQueue()
    private lateinit var biometricPrompt: BiometricPrompt
    private var initialAuthRetry: Job? = null
    private var activeAuthTimeout: Job? = null
    private var pendingAuthenticate: Job? = null

    val enrollmentMissing = _enrollmentMissing.asStateFlow()

    fun prepare() {
        val executor = ContextCompat.getMainExecutor(activity)
        biometricPrompt = BiometricPrompt(
            activity,
            executor,
            object : BiometricPrompt.AuthenticationCallback() {
                override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                    privacyCover.onPromptEnded()
                    if (!lockViewModel.uiState.value.isUnlocked) {
                        retryOrCloseAfterAuthError(errorCode)
                    } else if (authRequests.hasActive()) {
                        SystemAuthPolicy.errorText(errorCode)?.let { Toast.makeText(activity, it.text(activity), Toast.LENGTH_LONG).show() }
                        cancelActiveAuthRequest()
                    }
                }

                override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                    privacyCover.onPromptEnded()
                    initialAuthRetry?.cancel()
                    if (!lockViewModel.uiState.value.isUnlocked) {
                        lockViewModel.onInitialAuth(AuthState.Success)
                    } else if (authRequests.hasActive()) {
                        completeActiveAuthRequest()
                    }
                }
            },
        )
    }

    fun authenticate() {
        pendingAuthenticate?.cancel()
        pendingAuthenticate = activity.lifecycleScope.launch {
            try {
                activity.lifecycle.withResumed {
                    privacyCover.onPromptShown()
                    biometricPrompt.authenticate(buildPrompt(authRequests.activeRequiresConfirmation()))
                }
            } catch (_: LifecycleDestroyedException) {
            }
        }
    }

    private fun buildPrompt(requiresConfirmation: Boolean): BiometricPrompt.PromptInfo = BiometricPrompt.PromptInfo.Builder()
        .setTitle(activity.getString(R.string.settings_security_authentication))
        .setAllowedAuthenticators(SystemAuthPolicy.allowedAuthenticators)
        .setConfirmationRequired(requiresConfirmation)
        .build()

    fun refreshEnrollment(): Boolean {
        val canAuth = BiometricManager.from(activity).canAuthenticate(SystemAuthPolicy.allowedAuthenticators)
        val isEnrollmentMissing = SystemAuthPolicy.isEnrollmentMissing(canAuth)
        _enrollmentMissing.value = isEnrollmentMissing
        return !isEnrollmentMissing
    }

    fun openSettings() {
        runCatching { activity.startActivity(Intent(DevicePolicyManager.ACTION_SET_NEW_PASSWORD)) }.onFailure {
            activity.startActivity(Intent(Settings.ACTION_SECURITY_SETTINGS))
        }
    }

    fun requestAuth(auth: AuthRequest, onCancel: () -> Unit, onSuccess: () -> Unit) {
        if (lockViewModel.isAuthRequired() || auth == AuthRequest.Required) {
            if (refreshEnrollment()) {
                authRequests.enqueue(
                    requiresConfirmation = auth.requiresConfirmation,
                    onCancel = onCancel,
                    onSuccess = onSuccess,
                )?.let(::startAuthRequest)
            } else {
                openSettings()
                onCancel()
            }
        } else {
            onSuccess()
        }
    }

    fun cancel() {
        initialAuthRetry?.cancel()
        activeAuthTimeout?.cancel()
        pendingAuthenticate?.cancel()
        runCatching { biometricPrompt.cancelAuthentication() }
    }

    private fun retryOrCloseAfterAuthError(errorCode: Int) {
        val retryDelay = SystemAuthPolicy.initialRetryDelay(errorCode)
        if (retryDelay == null) {
            activity.finishAffinity()
            return
        }
        initialAuthRetry?.cancel()
        initialAuthRetry = activity.lifecycleScope.launch {
            if (retryDelay > Duration.ZERO) {
                delay(retryDelay)
            }
            if (!activity.isFinishing && !activity.isDestroyed) {
                lockViewModel.retryInitialAuth()
            }
        }
    }

    private fun startAuthRequest(request: PendingAuthRequest) {
        lockViewModel.requestAuth(requestId = request.id)
        activeAuthTimeout?.cancel()
        activeAuthTimeout = activity.lifecycleScope.launch {
            delay(SystemAuthPolicy.authRequestTimeout)
            val timedOut = authRequests.completeActive(request.id) ?: return@launch
            lockViewModel.completeAuthRequest(timedOut.id)
            timedOut.onCancel()
            runCatching { biometricPrompt.cancelAuthentication() }
            delay(SystemAuthPolicy.authRequestRestartDelay)
            activeAuthTimeout = null
            authRequests.startNext()?.let(::startAuthRequest)
        }
    }

    private fun completeActiveAuthRequest() {
        val request = authRequests.completeActive() ?: return
        activeAuthTimeout?.cancel()
        activeAuthTimeout = null
        if (lockViewModel.completeAuthRequest(request.id)) {
            request.onSuccess()
        }
        authRequests.startNext()?.let(::startAuthRequest)
    }

    private fun cancelActiveAuthRequest() {
        val request = authRequests.completeActive() ?: return
        activeAuthTimeout?.cancel()
        activeAuthTimeout = null
        lockViewModel.completeAuthRequest(request.id)
        request.onCancel()
        authRequests.startNext()?.let(::startAuthRequest)
    }
}

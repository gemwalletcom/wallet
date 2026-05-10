package com.gemwallet.android

import android.content.Intent
import android.os.Build
import android.provider.Settings
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.fragment.app.FragmentActivity
import androidx.lifecycle.lifecycleScope
import com.gemwallet.android.model.AuthRequest
import com.gemwallet.android.model.AuthState
import com.gemwallet.android.model.requiresConfirmation
import com.gemwallet.android.ui.R
import kotlin.time.Duration
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

internal class SystemAuthenticator(
    private val activity: FragmentActivity,
    private val viewModel: MainViewModel,
) {
    private val _enrollmentMissing = MutableStateFlow(false)
    private val authRequests = AuthRequestQueue()
    private lateinit var biometricPrompt: BiometricPrompt
    private var initialAuthRetry: Job? = null
    private var activeAuthTimeout: Job? = null

    val enrollmentMissing = _enrollmentMissing.asStateFlow()

    fun prepare() {
        val executor = ContextCompat.getMainExecutor(activity)
        biometricPrompt = BiometricPrompt(activity, executor, object : BiometricPrompt.AuthenticationCallback() {
            override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                if (viewModel.uiState.value.initialAuth != AuthState.Success) {
                    handleInitialAuthError(errorCode)
                } else if (authRequests.hasActive()) {
                    cancelActiveAuthRequest()
                }
            }

            override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                initialAuthRetry?.cancel()
                if (viewModel.uiState.value.initialAuth != AuthState.Success) {
                    viewModel.onInitialAuth(AuthState.Success)
                } else if (authRequests.hasActive()) {
                    completeActiveAuthRequest()
                }
            }
        })
    }

    fun authenticate() {
        biometricPrompt.authenticate(buildPrompt(authRequests.activeRequiresConfirmation()))
    }

    private fun buildPrompt(requiresConfirmation: Boolean): BiometricPrompt.PromptInfo =
        BiometricPrompt.PromptInfo.Builder()
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
        val intent = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            Intent(Settings.ACTION_BIOMETRIC_ENROLL).putExtra(
                Settings.EXTRA_BIOMETRIC_AUTHENTICATORS_ALLOWED,
                SystemAuthPolicy.allowedAuthenticators,
            )
        } else {
            Intent(Settings.ACTION_SECURITY_SETTINGS)
        }
        runCatching { activity.startActivity(intent) }.onFailure {
            activity.startActivity(Intent(Settings.ACTION_SECURITY_SETTINGS))
        }
    }

    fun requestAuth(auth: AuthRequest, onSuccess: () -> Unit) {
        if (!viewModel.isAuthRequired()) {
            onSuccess()
        } else if (refreshEnrollment()) {
            authRequests.enqueue(
                requiresConfirmation = auth.requiresConfirmation,
                onSuccess = onSuccess,
            )?.let(::startAuthRequest)
        } else {
            openSettings()
        }
    }

    fun cancel() {
        initialAuthRetry?.cancel()
        activeAuthTimeout?.cancel()
        runCatching { biometricPrompt.cancelAuthentication() }
    }

    private fun handleInitialAuthError(errorCode: Int) {
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
                viewModel.retryInitialAuth()
            }
        }
    }

    private fun startAuthRequest(request: PendingAuthRequest) {
        viewModel.requestAuth(requestId = request.id)
        activeAuthTimeout?.cancel()
        activeAuthTimeout = activity.lifecycleScope.launch {
            delay(SystemAuthPolicy.authRequestTimeout)
            val timedOut = authRequests.completeActive(request.id) ?: return@launch
            viewModel.completeAuthRequest(timedOut.id)
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
        if (viewModel.completeAuthRequest(request.id)) {
            request.onSuccess()
        }
        authRequests.startNext()?.let(::startAuthRequest)
    }

    private fun cancelActiveAuthRequest() {
        val request = authRequests.completeActive() ?: return
        activeAuthTimeout?.cancel()
        activeAuthTimeout = null
        viewModel.completeAuthRequest(request.id)
        authRequests.startNext()?.let(::startAuthRequest)
    }
}

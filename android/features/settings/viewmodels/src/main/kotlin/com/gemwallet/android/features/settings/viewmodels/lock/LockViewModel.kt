package com.gemwallet.android.features.settings.viewmodels.lock

import androidx.lifecycle.ViewModel
import com.gemwallet.android.application.KeystoreKeyChangedException
import com.gemwallet.android.application.security.cases.SecurityPreferences
import com.gemwallet.android.features.settings.viewmodels.lock.models.AuthState
import com.gemwallet.android.features.settings.viewmodels.lock.models.LockUIState
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.update
import java.util.concurrent.atomic.AtomicLong
import javax.inject.Inject

@HiltViewModel
class LockViewModel @Inject constructor(private val securityPreferences: SecurityPreferences, private val lockTimer: LockTimer) : ViewModel() {

    private val initialAuth = readInitialAuth()

    private val _uiState = MutableStateFlow(
        LockUIState(
            initialAuth = initialAuth,
            hasUnlockedApp = initialAuth == AuthState.Success,
        ),
    )
    val uiState: StateFlow<LockUIState> = _uiState.asStateFlow()

    private val activeAuthRequestId = AtomicLong(NoActiveAuthRequestId)

    fun isAuthRequired(): Boolean = securityPreferences.isLockEnabled()

    fun observeAuthRequired(): Flow<Boolean> = if (initialAuth == AuthState.KeyChanged) flowOf(true) else securityPreferences.getLockEnabled()

    fun requestAuth(requestId: Long) {
        activeAuthRequestId.set(requestId)
        _uiState.update { current ->
            current.copy(
                authState = AuthState.Required,
                authPromptRequest = current.authPromptRequest + 1,
            )
        }
    }

    fun retryInitialAuth() {
        _uiState.update { current ->
            if (current.initialAuth == AuthState.Success) {
                current
            } else {
                current.copy(
                    initialAuth = AuthState.Required,
                    authPromptRequest = current.authPromptRequest + 1,
                )
            }
        }
    }

    fun onInitialAuth(authState: AuthState) {
        _uiState.update { current ->
            if (current.initialAuth == AuthState.Success) {
                current
            } else {
                current.copy(
                    initialAuth = authState,
                    hasUnlockedApp = current.hasUnlockedApp || authState == AuthState.Success,
                )
            }
        }
    }

    fun completeAuthRequest(requestId: Long): Boolean {
        if (!activeAuthRequestId.compareAndSet(requestId, NoActiveAuthRequestId)) return false
        _uiState.update { it.copy(authState = null) }
        return true
    }

    fun onLeave() {
        lockTimer.onLeft()
    }

    fun onReturn(): Boolean {
        if (initialAuth == AuthState.KeyChanged || !lockTimer.shouldRelockOnReturn()) return false
        relock()
        return true
    }

    internal fun relock() {
        activeAuthRequestId.set(NoActiveAuthRequestId)
        _uiState.update { current ->
            current.copy(
                initialAuth = AuthState.Required,
                authState = null,
                authPromptRequest = current.authPromptRequest + 1,
            )
        }
    }

    private fun readInitialAuth(): AuthState = try {
        if (isAuthRequired()) AuthState.Required else AuthState.Success
    } catch (_: KeystoreKeyChangedException) {
        AuthState.KeyChanged
    }
}

private const val NoActiveAuthRequestId = -1L

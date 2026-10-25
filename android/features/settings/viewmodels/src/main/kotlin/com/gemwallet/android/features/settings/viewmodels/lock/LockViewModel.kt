package com.gemwallet.android.features.settings.viewmodels.lock

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.KeystoreKeyChangedException
import com.gemwallet.android.application.security.cases.KeystoreRecovery
import com.gemwallet.android.application.security.cases.SecurityPreferences
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.features.settings.viewmodels.lock.models.AuthState
import com.gemwallet.android.features.settings.viewmodels.lock.models.LockUIState
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.util.concurrent.atomic.AtomicLong
import javax.inject.Inject

@HiltViewModel
class LockViewModel @Inject constructor(
    private val securityPreferences: SecurityPreferences,
    private val lockTimer: LockTimer,
    private val keystoreRecovery: KeystoreRecovery,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val _uiState = MutableStateFlow(
        readInitialAuth().let { initialAuth ->
            LockUIState(
                initialAuth = initialAuth,
                hasUnlockedApp = initialAuth == AuthState.Success,
            )
        },
    )
    val uiState: StateFlow<LockUIState> = _uiState.asStateFlow()

    private val activeAuthRequestId = AtomicLong(NoActiveAuthRequestId)

    fun isAuthRequired(): Boolean = securityPreferences.isLockEnabled()

    @OptIn(ExperimentalCoroutinesApi::class)
    fun observeAuthRequired(): Flow<Boolean> = uiState.map { it.initialAuth.isKeystoreRecovery }
        .distinctUntilChanged()
        .flatMapLatest { isKeystoreRecovery -> if (isKeystoreRecovery) flowOf(true) else securityPreferences.getLockEnabled() }

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
            when (current.initialAuth) {
                AuthState.Success, AuthState.WalletsUnreadable -> current

                AuthState.ResetRequested -> current.copy(initialAuth = AuthState.WalletsUnreadable)

                AuthState.KeyChanged -> current.copy(authPromptRequest = current.authPromptRequest + 1)

                else -> current.copy(
                    initialAuth = AuthState.Required,
                    authPromptRequest = current.authPromptRequest + 1,
                )
            }
        }
    }

    fun onInitialAuth(authState: AuthState) {
        val current = _uiState.value.initialAuth
        if (current.isKeystoreRecovery) {
            if (authState == AuthState.Success) onKeystoreRecoveryAuthenticated(current)
            return
        }
        _uiState.update { state ->
            if (state.initialAuth == AuthState.Success) {
                state
            } else {
                state.copy(
                    initialAuth = authState,
                    hasUnlockedApp = state.hasUnlockedApp || authState == AuthState.Success,
                )
            }
        }
    }

    fun requestReset() {
        _uiState.update { current ->
            if (current.initialAuth == AuthState.WalletsUnreadable) {
                current.copy(initialAuth = AuthState.ResetRequested, authPromptRequest = current.authPromptRequest + 1)
            } else {
                current
            }
        }
    }

    fun dismissResetError() {
        _uiState.update { it.copy(resetError = null) }
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
        if (_uiState.value.initialAuth.isKeystoreRecovery || !lockTimer.shouldRelockOnReturn()) return false
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

    private fun onKeystoreRecoveryAuthenticated(state: AuthState) = viewModelScope.launch(ioDispatcher) {
        when (state) {
            AuthState.KeyChanged -> repairLockStores()
            AuthState.ResetRequested -> resetLocalData()
            else -> Unit
        }
    }

    private suspend fun repairLockStores() {
        if (!keystoreRecovery.repairLockStores()) {
            _uiState.update { it.copy(initialAuth = AuthState.WalletsUnreadable) }
            return
        }
        securityPreferences.setAuthRequired(true)
        _uiState.update { it.copy(initialAuth = AuthState.Success, hasUnlockedApp = true) }
    }

    private suspend fun resetLocalData() {
        runCatchingCancellable { keystoreRecovery.resetLocalData() }
            .onSuccess { _uiState.update { it.copy(isLocalDataReset = true) } }
            .onFailure { error ->
                Log.e("LockViewModel", "local data reset failed", error)
                _uiState.update { it.copy(initialAuth = AuthState.WalletsUnreadable, resetError = error.errorText()) }
            }
    }

    private fun readInitialAuth(): AuthState = try {
        if (isAuthRequired()) AuthState.Required else AuthState.Success
    } catch (_: KeystoreKeyChangedException) {
        AuthState.KeyChanged
    }
}

private const val NoActiveAuthRequestId = -1L

package com.gemwallet.android.features.settings.security.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.WalletPasswordProtection
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.GemConstants
import com.gemwallet.android.features.settings.security.viewmodels.models.LockPeriodOption
import com.gemwallet.android.ui.localization.stringRes

import com.gemwallet.android.ui.localization.text
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemListSection
import uniffi.gemstone.GemSecurityInput
import uniffi.gemstone.GemSettingsServiceInterface
import uniffi.gemstone.lockPeriodFromMinutes
import javax.inject.Inject

@HiltViewModel
class SecurityViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val passwordProtection: WalletPasswordProtection,
    private val settingsService: GemSettingsServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val error = MutableStateFlow<String?>(null)

    private val authRequired = MutableStateFlow(userConfig.authRequired())

    init {
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable {
                authRequired.value = passwordProtection.authenticationRequired() || userConfig.authRequired()
            }.onFailure { error.value = it.errorText().text(context) }
        }
    }

    val lockPeriods = GemConstants.lockPeriods.map { LockPeriodOption(it.minutes().toInt(), it.stringRes()) }

    val lockInterval = userConfig.getLockInterval()

    val sections = combine(authRequired, userConfig.getLockInterval(), userConfig.isHideBalances()) { authRequired, lockInterval, hideBalances ->
        sections(authRequired, lockInterval, hideBalances)
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, sections(authRequired.value, lockPeriodFromMinutes(null).minutes().toInt(), false))

    private fun sections(authRequired: Boolean, lockInterval: Int, hideBalances: Boolean): List<GemListSection> = settingsService.securitySections(
        GemSecurityInput(
            authenticationEnabled = authRequired,
            authenticationName = null,
            lockPeriod = context.getString(lockPeriodFromMinutes(lockInterval.toUInt()).stringRes()),
            privacyLockEnabled = false,
            privacyLockSupported = false,
            hideBalanceEnabled = hideBalances,
        ),
    )

    val isUpdatingAuthentication = MutableStateFlow(false)

    fun setAuthRequired(required: Boolean) {
        if (isUpdatingAuthentication.value) return
        isUpdatingAuthentication.value = true
        viewModelScope.launch(ioDispatcher) {
            try {
                runCatchingCancellable {
                    passwordProtection.setAuthenticationRequired(required)
                    userConfig.setAuthRequired(required)
                    authRequired.value = required
                }.onFailure { error.value = it.errorText().text(context) }
            } finally {
                isUpdatingAuthentication.value = false
            }
        }
    }

    fun clearError() {
        error.value = null
    }

    fun setLockInterval(minutes: Int) = viewModelScope.launch(ioDispatcher) {
        userConfig.setLockInterval(minutes)
    }

    fun setHideBalances() {
        viewModelScope.launch(ioDispatcher) {
            userConfig.hideBalances()
        }
    }
}

package com.gemwallet.android.features.settings.security.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.features.settings.security.viewmodels.localization.stringRes
import com.gemwallet.android.features.settings.security.viewmodels.models.LockPeriodOption
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
import uniffi.gemstone.lockPeriods
import javax.inject.Inject

@HiltViewModel
class SecurityViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val settingsService: GemSettingsServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val authRequired = MutableStateFlow(userConfig.authRequired())

    val lockPeriods = lockPeriods().map { LockPeriodOption(it.minutes().toInt(), it.stringRes()) }

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

    fun setAuthRequired(required: Boolean) {
        userConfig.setAuthRequired(required)
        authRequired.value = required
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

package com.gemwallet.android.features.settings.security.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.lockPeriodFromMinutes
import javax.inject.Inject
import uniffi.gemstone.GemSecuritySection
import uniffi.gemstone.GemSettingsServiceInterface

@HiltViewModel
class SecurityViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val settingsService: GemSettingsServiceInterface,
) : ViewModel() {

    fun sections(authenticationEnabled: Boolean): List<GemSecuritySection> = settingsService.securitySections(authenticationEnabled)

    val isHideBalances = userConfig.isHideBalances()
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val lockInterval = userConfig.getLockInterval()
        .stateIn(viewModelScope, SharingStarted.Eagerly, lockPeriodFromMinutes(null).minutes().toInt())

    fun authRequired(): Boolean {
        return userConfig.authRequired()
    }

    fun setAuthRequired(required: Boolean) {
        userConfig.setAuthRequired(required)
    }

    fun setLockInterval(minutes: Int) = viewModelScope.launch(Dispatchers.IO) {
        userConfig.setLockInterval(minutes)
    }

    fun setHideBalances() {
        viewModelScope.launch(Dispatchers.IO) {
            userConfig.hideBalances()
        }
    }
}
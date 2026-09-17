package com.gemwallet.android.features.settings.security.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.gemstone.di.IoDispatcher
import com.gemwallet.android.features.settings.security.viewmodels.localization.stringRes
import com.gemwallet.android.features.settings.security.viewmodels.models.LockPeriodOption
import com.gemwallet.android.features.settings.security.viewmodels.models.SecurityRowUIModel
import com.gemwallet.android.features.settings.security.viewmodels.models.uiModel
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemSettingsServiceInterface
import uniffi.gemstone.lockPeriodFromMinutes
import uniffi.gemstone.lockPeriods

@HiltViewModel
class SecurityViewModel @Inject constructor(
    private val userConfig: UserConfig,
    private val settingsService: GemSettingsServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val authRequired = MutableStateFlow(userConfig.authRequired())
    private val lockPeriods = lockPeriods().map { LockPeriodOption(it.minutes().toInt(), it.stringRes()) }

    val rows = combine(authRequired, userConfig.getLockInterval(), userConfig.isHideBalances()) { authRequired, lockInterval, hideBalances ->
        rows(authRequired, lockInterval, hideBalances)
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, rows(authRequired.value, lockPeriodFromMinutes(null).minutes().toInt(), false))

    private fun rows(authRequired: Boolean, lockInterval: Int, hideBalances: Boolean): List<List<SecurityRowUIModel>> =
        settingsService.securitySections(authRequired).map { section ->
            section.rows.mapNotNull { it.uiModel(context, authRequired, lockInterval, hideBalances, lockPeriods) }
        }

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

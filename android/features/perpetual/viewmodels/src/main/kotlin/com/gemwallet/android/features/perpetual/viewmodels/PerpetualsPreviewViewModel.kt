package com.gemwallet.android.features.perpetual.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualPositions
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.config.showPerpetuals
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@HiltViewModel
class PerpetualsPreviewViewModel @Inject constructor(
    userConfig: UserConfig,
    getSession: GetSession,
    getPositions: GetPerpetualPositions,
) : ViewModel() {

    val showPerpetuals = userConfig.showPerpetuals(getSession())
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val positions = getPositions.getPerpetualPositions()
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
}

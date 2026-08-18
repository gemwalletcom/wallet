package com.gemwallet.android.features.asset.viewmodels.details.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.device.coordinators.EnableDevicePush
import com.gemwallet.android.application.pricealerts.coordinators.GetAssetPriceAlertState
import com.gemwallet.android.application.pricealerts.coordinators.GetPriceAlerts
import com.gemwallet.android.application.pricealerts.coordinators.SetAssetPriceAlertEnabled
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class AssetPriceAlertsViewModel @Inject constructor(
    getSession: GetSession,
    savedStateHandle: SavedStateHandle,
    getAssetPriceAlertState: GetAssetPriceAlertState,
    getPriceAlerts: GetPriceAlerts,
    private val setAssetPriceAlertEnabled: SetAssetPriceAlertEnabled,
    private val enableDevicePush: EnableDevicePush,
) : ViewModel() {

    private val assetId = savedStateHandle.requireAssetId()

    private val observedAssetId = assetId.takeIf { getSession().value?.wallet != null }

    private val enabledState: Flow<Boolean?> = observedAssetId
        ?.let { getAssetPriceAlertState.isAssetPriceAlertEnabled(it) }
        ?: flowOf(null)

    val isEnabled: StateFlow<Boolean?> = enabledState
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val alertsCount = getPriceAlerts(assetId)
        .map { it.size }
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0)

    fun toggle(assetId: AssetId) = viewModelScope.launch {
        val enabled = isEnabled.value ?: return@launch
        setAssetPriceAlertEnabled(assetId, !enabled)
    }

    fun onPushNotificationGranted() = viewModelScope.launch(Dispatchers.IO) {
        enableDevicePush()
    }
}

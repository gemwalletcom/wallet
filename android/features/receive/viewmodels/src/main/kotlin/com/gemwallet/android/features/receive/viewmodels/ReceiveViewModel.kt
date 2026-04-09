package com.gemwallet.android.features.receive.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.receive.coordinators.GetReceiveAssetInfo
import com.gemwallet.android.application.receive.coordinators.SetAssetVisible
import com.gemwallet.android.ext.toAssetId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ReceiveViewModel @Inject constructor(
    private val getReceiveAssetInfo: GetReceiveAssetInfo,
    private val setAssetVisible: SetAssetVisible,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val assetId = savedStateHandle.getStateFlow("assetId", "").map { it.toAssetId() }

    val asset = assetId
        .filterNotNull()
        .flatMapLatest { getReceiveAssetInfo(it) }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun setVisible() = viewModelScope.launch {
        val assetId = asset.value?.asset?.id ?: return@launch
        setAssetVisible(assetId)
    }
}

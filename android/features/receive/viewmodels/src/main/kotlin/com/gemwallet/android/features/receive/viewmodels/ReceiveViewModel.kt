package com.gemwallet.android.features.receive.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.coordinators.GetChainAssetInfo
import com.gemwallet.android.application.receive.coordinators.GetReceiveAssetInfo
import com.gemwallet.android.application.receive.coordinators.SetAssetVisible
import com.gemwallet.android.ext.type
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetSubtype
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import java.math.BigInteger
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ReceiveViewModel @Inject constructor(
    private val getReceiveAssetInfo: GetReceiveAssetInfo,
    private val getChainAssetInfo: GetChainAssetInfo,
    private val setAssetVisible: SetAssetVisible,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val assetId = MutableStateFlow(savedStateHandle.requireAssetId(RouteArgument.AssetId))

    val asset = assetId
        .flatMapLatest { getReceiveAssetInfo(it) }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val showFeeWarning = MutableStateFlow(false)

    init {
        checkFeeWarning()
    }

    private fun checkFeeWarning() = viewModelScope.launch {
        val id = assetId.value
        if (id.type() != AssetSubtype.TOKEN || id.chain == Chain.HyperCore) return@launch
        val info = getChainAssetInfo(id).filterNotNull().firstOrNull() ?: return@launch
        showFeeWarning.value = info.feeAssetInfo.balance.balance.available.toBigInteger() == BigInteger.ZERO
    }

    fun setVisible() = viewModelScope.launch {
        val assetId = asset.value?.asset?.id ?: return@launch
        setAssetVisible(assetId)
    }
}

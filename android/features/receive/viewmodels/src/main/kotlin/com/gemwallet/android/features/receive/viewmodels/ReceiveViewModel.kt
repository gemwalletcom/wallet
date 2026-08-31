package com.gemwallet.android.features.receive.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.SyncAssetInfo
import com.gemwallet.android.application.receive.cases.GetReceiveAssetInfo
import com.gemwallet.android.application.receive.cases.SetAssetVisible
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemMemoWarning
import uniffi.gemstone.GemReceiveService
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ReceiveViewModel @Inject constructor(
    private val getReceiveAssetInfo: GetReceiveAssetInfo,
    private val setAssetVisible: SetAssetVisible,
    private val syncAssetInfo: SyncAssetInfo,
    private val receiveService: GemReceiveService,
    getSession: GetSession,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val sourceAssetId = savedStateHandle.requireAssetId(RouteArgument.AssetId)
    private val selectedAssetId = MutableStateFlow(sourceAssetId)
    private val session = getSession()

    val asset = selectedAssetId
        .flatMapLatest { getReceiveAssetInfo(it) }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val networkAssetIds = combine(
        asset.filterNotNull().filter { it.asset.id == sourceAssetId },
        session.filterNotNull(),
    ) { assetInfo, session ->
        (listOf(assetInfo.asset.id) + assetInfo.associations.map { it.assetId })
            .filter { session.wallet.getAccount(it) != null }
            .distinct()
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, listOf(sourceAssetId))

    init {
        viewModelScope.launch(Dispatchers.IO) {
            syncAssetInfo.syncAssetInfo(sourceAssetId, session.filterNotNull().first().wallet)
        }
    }

    fun memoWarning(chain: Chain): GemMemoWarning = receiveService.memoWarning(chain.string)

    fun selectAsset(assetId: AssetId) {
        selectedAssetId.value = assetId
    }

    fun setVisible() = viewModelScope.launch {
        val assetId = asset.value?.asset?.id ?: return@launch
        setAssetVisible(assetId)
    }
}

package com.gemwallet.android.features.receive.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.receive.cases.GetReceiveAssetInfo
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AssetId
import dagger.assisted.Assisted
import dagger.assisted.AssistedFactory
import dagger.assisted.AssistedInject
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
import com.wallet.core.primitives.Chain
import uniffi.gemstone.GemMemoWarning
import uniffi.gemstone.GemReceiveServiceInterface

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel(assistedFactory = ReceiveViewModel.Factory::class)
class ReceiveViewModel @AssistedInject constructor(
    @Assisted private val sourceAssetId: AssetId,
    private val getReceiveAssetInfo: GetReceiveAssetInfo,
    private val service: GemReceiveServiceInterface,
    getSession: GetSession,
) : ViewModel() {

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
        service.networkAssetIds(
            assetInfo.asset.id.toIdentifier(),
            assetInfo.associations.map { it.assetId.toIdentifier() },
            session.wallet.toJson(),
        ).map { it.toAssetId()!! }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, listOf(sourceAssetId))

    init {
        viewModelScope.launch(Dispatchers.IO) {
            val associations = networkAssetIds.first { it.size > 1 }.filter { it != sourceAssetId }
            runCatchingCancellable { service.syncMissingAssets(associations.map { it.toIdentifier() }) }
        }
    }

    fun memoWarning(chain: Chain): GemMemoWarning = service.memoWarning(chain.string)

    fun selectAsset(assetId: AssetId) {
        selectedAssetId.value = assetId
    }

    @AssistedFactory
    interface Factory {
        fun create(assetId: AssetId): ReceiveViewModel
    }

    fun setVisible() = viewModelScope.launch(Dispatchers.IO) {
        val assetId = asset.value?.asset?.id ?: return@launch
        val wallet = session.filterNotNull().first().wallet
        service.enableAsset(wallet.id.id, assetId.toIdentifier())
    }
}

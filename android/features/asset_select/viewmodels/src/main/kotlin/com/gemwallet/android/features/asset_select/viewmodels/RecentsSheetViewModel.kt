package com.gemwallet.android.features.asset_select.viewmodels

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.application.asset_select.cases.ClearRecentAssets
import com.gemwallet.android.application.asset_select.cases.GetRecentAssets
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.asset_select.viewmodels.models.RecentsSheetUIModel
import com.gemwallet.android.model.AssetFilter
import com.gemwallet.android.model.RecentAsset
import com.gemwallet.android.model.RecentAssetsRequest
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import dagger.hilt.android.lifecycle.HiltViewModel
import uniffi.gemstone.GemAssetConfigService
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class RecentsSheetViewModel @Inject constructor(
    private val getRecentAssets: GetRecentAssets,
    private val clearRecentAssets: ClearRecentAssets,
    private val assetConfig: GemAssetConfigService,
) : ViewModel() {

    val query = TextFieldState()

    private val config = MutableStateFlow<Config?>(null)

    val visible: StateFlow<Boolean> = config
        .map { it != null }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val uiModel: StateFlow<RecentsSheetUIModel> = config
        .filterNotNull()
        .flatMapLatest { config ->
            combine(
                getRecentAssets(RecentAssetsRequest(types = config.types, filters = config.filters, limit = 0)),
                snapshotFlow { query.text.toString() },
                ::buildUIModel,
            )
        }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, RecentsSheetUIModel.Empty)

    fun show(filters: Set<AssetFilter> = emptySet(), types: List<RecentType> = RecentType.entries) {
        query.clearText()
        config.value = Config(filters, types)
    }

    fun dismiss() {
        config.value = null
    }

    fun onClear() {
        val types = config.value?.types ?: RecentType.entries
        viewModelScope.launch(Dispatchers.IO) { clearRecentAssets(types) }
    }

    private fun buildUIModel(items: List<RecentAsset>, searchText: String): RecentsSheetUIModel {
        val matching = assetConfig.matchingAssets(items.map { it.asset.toGem() }, searchText)
            .map { it.toPrimitives().id.toIdentifier() }
            .toSet()
        val filtered = items.filter { it.asset.id.toIdentifier() in matching }
        return RecentsSheetUIModel(
            items = filtered.toImmutableList(),
            hasAnyRecents = items.isNotEmpty(),
            searchActive = searchText.isNotBlank(),
        )
    }

    private data class Config(
        val filters: Set<AssetFilter>,
        val types: List<RecentType>,
    )
}

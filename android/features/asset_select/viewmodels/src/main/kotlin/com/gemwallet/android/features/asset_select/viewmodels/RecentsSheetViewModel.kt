package com.gemwallet.android.features.asset_select.viewmodels

import android.util.Log
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.data.services.gemstone.assets.RecentAssetsService
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.asset_select.viewmodels.models.RecentsSheetUIModel
import com.gemwallet.android.model.RecentAsset
import com.gemwallet.android.model.RecentAssetsRequest
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.RecentActivityType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.CoroutineDispatcher
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
import uniffi.gemstone.GemAssetFilter
import uniffi.gemstone.GemRecentActivityServiceInterface
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class RecentsSheetViewModel @Inject constructor(
    private val recentAssetsService: RecentAssetsService,
    private val recentActivityService: GemRecentActivityServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
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
                recentAssetsService.getRecentAssets(RecentAssetsRequest(types = config.types, filters = config.filters, limit = 0)),
                snapshotFlow { query.text.toString() },
                ::buildUIModel,
            )
        }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, RecentsSheetUIModel.Empty)

    fun show(filters: Set<GemAssetFilter> = emptySet(), types: List<RecentActivityType> = RecentActivityType.entries) {
        query.clearText()
        config.value = Config(filters, types)
    }

    fun dismiss() {
        config.value = null
    }

    fun onClear() {
        val types = config.value?.types ?: RecentActivityType.entries
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { recentActivityService.clear(types.map { it.toGem() }) }
                .onFailure { Log.e(TAG, "clearing recents failed", it) }
        }
    }

    private fun buildUIModel(items: List<RecentAsset>, searchText: String): RecentsSheetUIModel {
        val state = recentActivityService.viewState(items.map { it.asset.toGem() }, searchText)
        val matching = state.matchingAssetIds.toSet()
        return RecentsSheetUIModel(
            items = items.filter { it.asset.id.toIdentifier() in matching }.toImmutableList(),
            sections = state.sections,
        )
    }

    private data class Config(val filters: Set<GemAssetFilter>, val types: List<RecentActivityType>)
}

private const val TAG = "RecentsSheet"

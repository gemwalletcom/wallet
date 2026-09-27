package com.gemwallet.android.features.assets.viewmodels.select

import android.util.Log
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.values.toQueryFilter
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.data.services.store.queries.RecentActivityQuery
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ui.format.gemDay
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.RecentActivityType
import com.wallet.core.primitives.RecentAsset
import dagger.hilt.android.lifecycle.HiltViewModel
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
import uniffi.gemstone.GemRecentsViewState
import java.time.Instant
import java.time.ZoneId
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class RecentsViewModel @Inject constructor(
    private val getCurrentWalletId: GetCurrentWalletId,
    private val recentActivityQuery: RecentActivityQuery,
    private val recentActivityService: GemRecentActivityServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    val query = TextFieldState()

    private val config = MutableStateFlow<Config?>(null)

    val visible: StateFlow<Boolean> = config
        .map { it != null }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val viewState: StateFlow<GemRecentsViewState> = config
        .filterNotNull()
        .flatMapLatest { config ->
            combine(
                getCurrentWalletId().flatMapLatest { recentActivityQuery(it, config.types, config.filters.map { filter -> filter.toQueryFilter() }.toSet(), limit = 0) },
                snapshotFlow { query.text.toString() },
                ::viewState,
            )
        }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, recentActivityService.viewState(emptyList(), emptyList(), ""))

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

    private fun viewState(items: List<RecentAsset>, searchText: String): GemRecentsViewState {
        val zone = ZoneId.systemDefault()
        return recentActivityService.viewState(items.map { it.toGem() }, items.map { Instant.ofEpochMilli(it.createdAt).atZone(zone).toLocalDate().gemDay() }, searchText)
    }

    private data class Config(val filters: Set<GemAssetFilter>, val types: List<RecentActivityType>)
}

private const val TAG = "RecentsSheet"

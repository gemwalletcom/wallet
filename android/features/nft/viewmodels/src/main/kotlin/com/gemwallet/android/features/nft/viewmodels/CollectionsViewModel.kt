package com.gemwallet.android.features.nft.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.NFTQuery
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ToastEmitter
import com.gemwallet.android.ui.models.ToastEmitterImpl
import com.gemwallet.android.ui.models.ToastMessage
import com.gemwallet.android.ui.models.toUIModels
import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChangedBy
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemNftList
import uniffi.gemstone.GemNftListScreen
import uniffi.gemstone.GemNftServiceInterface
import uniffi.gemstone.loadError
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class CollectionsViewModel @Inject constructor(
    private val nftService: GemNftServiceInterface,
    nftQuery: NFTQuery,
    getSession: GetSession,
    savedStateHandle: SavedStateHandle,
    @param:ApplicationContext private val context: Context,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel(),
    ToastEmitter by ToastEmitterImpl() {

    val list: GemNftList = savedStateHandle.nftList()

    private val _isRefreshing = MutableStateFlow(false)
    val isRefreshing: StateFlow<Boolean> = _isRefreshing

    private val loadState = MutableStateFlow<GemLoadState>(GemLoadState.Loading)

    private val session = getSession()

    val walletId: StateFlow<WalletId?> = session
        .map { it?.wallet?.id }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private var lastSyncedWalletId: WalletId? = null

    private val collectionId = savedStateHandle.nftCollectionId()

    private val nftData: StateFlow<List<NFTData>> = session
        .filterNotNull()
        .distinctUntilChangedBy { it.wallet.id }
        .flatMapLatest { nftQuery(it.wallet.id.id, collectionId) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val screen: StateFlow<GemNftListScreen> = nftData
        .map { data -> nftService.listScreen(data.map { it.toGem() }, list) }
        .flowOn(Dispatchers.Default)
        .stateIn(viewModelScope, SharingStarted.Eagerly, nftService.listScreen(emptyList(), list))

    val title: StateFlow<String> = screen
        .map { it.title.string(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val showReceiveAction: StateFlow<Boolean> = screen
        .map { it.offersReceive }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val collections = screen
        .map { it.items.toUIModels() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val errorRow: StateFlow<GemListRow?> = combine(loadState, screen) { state, screen ->
        loadError(state, screen.hasContent)?.let { GemListRow.Error(it) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val unverifiedListItem: StateFlow<ListItemModel?> = screen
        .map { screen -> screen.unverifiedRow?.let { ListItemModel(title = context.getString(R.string.asset_verification_unverified), subtitle = it.countText) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun syncIfNeeded() {
        if (!screen.value.syncsOnAppear) return
        val current = walletId.value ?: return
        if (current == lastSyncedWalletId) return
        lastSyncedWalletId = current
        viewModelScope.launch(ioDispatcher) {
            sync()
        }
    }

    fun refresh() {
        viewModelScope.launch(ioDispatcher) {
            _isRefreshing.update { true }
            try {
                sync()
            } finally {
                _isRefreshing.update { false }
            }
        }
    }

    private suspend fun sync() {
        val result = nftService.refresh(screen.value.hasContent)
        loadState.update { result.state }
        result.toast?.let { emitToast(ToastMessage(it.errorText().text(context), R.drawable.ic_error)) }
    }
}

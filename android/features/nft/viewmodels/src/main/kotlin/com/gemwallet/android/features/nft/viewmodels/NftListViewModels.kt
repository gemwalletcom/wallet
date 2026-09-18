package com.gemwallet.android.features.nft.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.nft.cases.GetNftCollections
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.nft.viewmodels.localization.stringRes
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.models.toUIModels
import com.wallet.core.primitives.NFTData
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemNftList
import uniffi.gemstone.GemNftServiceInterface

@HiltViewModel
class NftListViewModels @Inject constructor(
    private val nftService: GemNftServiceInterface,
    getNftCollections: GetNftCollections,
    getSession: GetSession,
    savedStateHandle: SavedStateHandle,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val list: GemNftList = savedStateHandle.nftList()

    val title: String = context.getString(list.stringRes())

    val showReceiveAction: Boolean = list != GemNftList.UNVERIFIED

    private val _isRefreshing = MutableStateFlow(false)
    val isRefreshing: StateFlow<Boolean> = _isRefreshing

    private val session = getSession()

    val walletId: StateFlow<WalletId?> = session
        .map { it?.wallet?.id }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private var lastSyncedWalletId: WalletId? = null

    private val nftData: StateFlow<List<NFTData>> = getNftCollections(savedStateHandle.nftCollectionId())
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val collections = nftData
        .map { data -> nftService.listItems(data.map { it.toGem() }, list).toUIModels() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val unverifiedListItem: StateFlow<ListItemModel?> = nftData
        .map { data ->
            nftService.unverifiedRow(data.map { it.toGem() }, list)
                ?.let { ListItemModel(title = context.getString(R.string.asset_verification_unverified), subtitle = it.countText) }
        }
        .flowOn(Dispatchers.Default)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun syncIfNeeded() {
        if (list == GemNftList.COLLECTION) return
        val current = walletId.value ?: return
        if (current == lastSyncedWalletId) return
        lastSyncedWalletId = current
        viewModelScope.launch(Dispatchers.IO) {
            sync()
        }
    }

    fun refresh() {
        viewModelScope.launch(Dispatchers.IO) {
            _isRefreshing.update { true }
            try {
                sync()
            } finally {
                _isRefreshing.update { false }
            }
        }
    }

    private suspend fun sync() {
        runCatchingCancellable { nftService.sync() }
            .onFailure { Log.e(TAG, "nft collections sync failed", it) }
    }

    private companion object {
        const val TAG = "NftList"
    }
}

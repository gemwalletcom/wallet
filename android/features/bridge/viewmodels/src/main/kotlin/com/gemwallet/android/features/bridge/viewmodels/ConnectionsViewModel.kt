package com.gemwallet.android.features.bridge.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet_connect.cases.PairWalletConnect
import com.gemwallet.android.application.wallet_connect.cases.SyncWalletConnectSessions
import com.gemwallet.android.data.services.store.queries.ConnectionsQuery
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.bridge.viewmodels.model.rowUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemImage
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.components.list_item.ListItemSymbol
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ListSection
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemWalletConnectServiceInterface
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class ConnectionsViewModel @Inject constructor(
    connectionsQuery: ConnectionsQuery,
    private val syncWalletConnectSessions: SyncWalletConnectSessions,
    private val pairWalletConnect: PairWalletConnect,
    private val service: GemWalletConnectServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val view = connectionsQuery()
        .mapLatest { connections -> service.connectionsView(connections.map { it.toGem() }) }
        .stateIn(viewModelScope, SharingStarted.Companion.Lazily, null)

    val sections = view
        .map { view ->
            view?.sections.orEmpty().map { section ->
                ListSection(id = section.title, title = section.title, items = section.connections.map { it.rowUIModel() })
            }
        }
        .stateIn(viewModelScope, SharingStarted.Companion.Lazily, emptyList())

    val docsUrl = view
        .map { it?.docsUrl }
        .stateIn(viewModelScope, SharingStarted.Companion.Lazily, null)

    init {
        viewModelScope.launch(ioDispatcher) { syncWalletConnectSessions.syncSessions() }
    }

    val pasteListItem = ListItemModel(title = context.getString(R.string.common_paste), image = ListItemImage.Symbol(ListItemSymbol.Paste))
    val scanListItem = ListItemModel(title = context.getString(R.string.wallet_scan_qr_code), image = ListItemImage.Symbol(ListItemSymbol.QrScanner))

    fun addPairing(uri: String, onSuccess: () -> Unit, onError: (String) -> Unit) {
        viewModelScope.launch(ioDispatcher) {
            pairWalletConnect.pair(
                uri = uri,
                onSuccess = onSuccess,
                onError = { onError(it.text(context)) },
            )
        }
    }
}

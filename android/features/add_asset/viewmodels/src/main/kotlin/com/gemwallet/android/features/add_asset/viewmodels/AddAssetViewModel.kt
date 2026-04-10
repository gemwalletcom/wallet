package com.gemwallet.android.features.add_asset.viewmodels

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.add_asset.coordinators.AddCustomToken
import com.gemwallet.android.application.add_asset.coordinators.GetAvailableTokenChains
import com.gemwallet.android.application.add_asset.coordinators.ObserveToken
import com.gemwallet.android.application.add_asset.coordinators.SearchCustomToken
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.ext.filter
import com.gemwallet.android.features.add_asset.viewmodels.models.AddAssetUIState
import com.gemwallet.android.features.add_asset.viewmodels.models.TokenSearchState
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import uniffi.gemstone.Explorer
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AddAssetViewModel @Inject constructor(
    private val searchCustomToken: SearchCustomToken,
    private val observeToken: ObserveToken,
    private val addCustomToken: AddCustomToken,
    getAvailableTokenChains: GetAvailableTokenChains,
    private val getCurrentBlockExplorer: GetCurrentBlockExplorer,
) : ViewModel() {

    private val state = MutableStateFlow(State())
    val uiState = state.map { it.toUIState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, AddAssetUIState())

    val chainFilter = TextFieldState()

    val availableChains = getAvailableTokenChains()
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val chains = snapshotFlow { chainFilter.text }.combine(availableChains) { query, availableChains ->
        availableChains?.filter(query.toString().lowercase()) ?: emptyList()
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val defaultChain = availableChains.map {
        it.let {
            it?.firstOrNull { chain -> chain == Chain.Ethereum } ?: it?.firstOrNull() ?: Chain.Ethereum
        }
    }
    private val chain = MutableStateFlow<Chain?>(null)
    val selectedChain = defaultChain.combine(chain) {defaultChain, chain ->
        chain ?: defaultChain
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, Chain.Ethereum)

    val addressState = mutableStateOf("")
    val addressQuery = snapshotFlow { addressState.value }

    val searchState = addressQuery.combine(selectedChain) { address, chain ->
        Pair(address, chain)
    }.flatMapLatest {
        val (address, chain) = it
        flow {
            if (address.isEmpty()) {
                emit(TokenSearchState.Idle)
                return@flow
            }

            emit(TokenSearchState.Loading)
            emit(searchToken(searchCustomToken, observeToken, chain, address))
        }
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, TokenSearchState.Idle)

    val token = combine(addressQuery, selectedChain) { address, chain ->
        Pair(address.trim(), chain)
    }
    .flatMapLatest {
        val (address, chain) = it
        if (address.isEmpty()) {
            return@flatMapLatest flowOf(null)
        }
        observeToken(AssetId(chain, address))
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val explorerLink = token.combine(selectedChain) { token, chain ->
        val tokenId = token?.id?.tokenId ?: return@combine null
        val explorerName = getCurrentBlockExplorer.getCurrentBlockExplorer(chain)
        val url = Explorer(chain.string).getTokenUrl(explorerName, tokenId) ?: return@combine null
        BlockExplorerLink(name = explorerName, link = url)
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun onQrScan() {
        state.update { it.copy(isQrScan = true) }
    }

    fun cancelScan() {
        state.update {
            it.copy(
                isQrScan = false,
            )
        }
    }

    fun setQrData(data: String) {
        addressState.value = data
        state.update { it.copy(isQrScan = false) }
    }

    fun selectChain() {
        state.update { it.copy(isSelectChain = true) }
    }

    fun cancelSelectChain() {
        state.update { it.copy(isSelectChain = false) }
    }

    fun setChain(chain: Chain) {
        this.chain.update { chain }
        state.update { it.copy(isSelectChain = false) }
    }

    fun addAsset(onFinish: () -> Unit) = viewModelScope.launch {
        onFinish()
        async(Dispatchers.IO) {
            addCustomToken(selectedChain.value, token.value?.id ?: return@async)
        }.await()
    }

    private data class State(
        val isQrScan: Boolean = false,
        val isSelectChain: Boolean = false,
    ) {
        fun toUIState(): AddAssetUIState {
            return AddAssetUIState(
                scene = when {
                    isQrScan -> AddAssetUIState.Scene.QrScanner
                    isSelectChain -> AddAssetUIState.Scene.SelectChain
                    else -> AddAssetUIState.Scene.Form
                },
            )
        }
    }
}

suspend fun searchToken(
    searchCustomToken: SearchCustomToken,
    observeToken: ObserveToken,
    chain: Chain,
    address: String,
): TokenSearchState {
    val assetId = AssetId(chain, address)
    return try {
        searchCustomToken(assetId)
        val found = observeToken(assetId).firstOrNull() != null
        if (found) TokenSearchState.Idle else TokenSearchState.Error
    } catch (_: Exception) {
        TokenSearchState.Error
    }
}

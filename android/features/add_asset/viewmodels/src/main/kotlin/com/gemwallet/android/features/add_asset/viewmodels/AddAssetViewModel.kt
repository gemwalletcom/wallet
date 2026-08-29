package com.gemwallet.android.features.add_asset.viewmodels

import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.toIdentifier
import uniffi.gemstone.defaultTokenChain
import uniffi.gemstone.GemChainService
import uniffi.gemstone.GemExplorerService
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.add_asset.cases.AddCustomToken
import com.gemwallet.android.application.add_asset.cases.GetAvailableTokenChains
import com.gemwallet.android.application.add_asset.cases.ObserveToken
import com.gemwallet.android.application.add_asset.cases.SearchCustomToken
import com.gemwallet.android.ext.checksumAddress
import com.gemwallet.android.features.add_asset.viewmodels.models.AddAssetUIState
import com.gemwallet.android.features.add_asset.viewmodels.models.TokenSearchState
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
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
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AddAssetViewModel @Inject constructor(
    private val searchCustomToken: SearchCustomToken,
    private val observeToken: ObserveToken,
    private val addCustomToken: AddCustomToken,
    getAvailableTokenChains: GetAvailableTokenChains,
    private val explorerService: GemExplorerService,
    private val chainService: GemChainService,
) : ViewModel() {

    private val state = MutableStateFlow(State())
    val uiState = state.map { it.toUIState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, AddAssetUIState())

    val chainFilter = TextFieldState()

    val availableChains = getAvailableTokenChains()
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val chains = snapshotFlow { chainFilter.text }.combine(availableChains) { query, availableChains ->
        availableChains?.let { chainService.getMatchingChains(it.map { chain -> chain.string }, query.toString()).mapNotNull { chain -> chain.toChain() } } ?: emptyList()
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val defaultChain = availableChains.map { chains ->
        defaultTokenChain(chains.orEmpty().map { it.string })?.toChain() ?: Chain.Ethereum
    }
    private val chain = MutableStateFlow<Chain?>(null)
    val selectedChain = defaultChain.combine(chain) {defaultChain, chain ->
        chain ?: defaultChain
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, Chain.Ethereum)

    val addressState = mutableStateOf("")
    val addressQuery = snapshotFlow { addressState.value }

    private val customAssetId = addressQuery.combine(selectedChain) { address, chain ->
        AssetId(chain, chain.checksumAddress(address))
    }

    val searchState = customAssetId.flatMapLatest { assetId ->
        flow {
            if (assetId.tokenId.isNullOrEmpty()) {
                emit(TokenSearchState.Idle)
                return@flow
            }

            emit(TokenSearchState.Loading)
            emit(searchToken(searchCustomToken, observeToken, assetId))
        }
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, TokenSearchState.Idle)

    val token = customAssetId
    .flatMapLatest { assetId ->
        if (assetId.tokenId.isNullOrEmpty()) {
            return@flatMapLatest flowOf(null)
        }
        observeToken(assetId)
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val buttonState = combine(searchState, token, uiState) { searchState, token, uiState ->
        buttonState(
            enabled = searchState is TokenSearchState.Idle && token != null,
            loading = uiState.isLoading,
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Disabled)

    val explorerLink = token.combine(selectedChain) { token, chain ->
        val tokenId = token?.id?.tokenId ?: return@combine null
        val link = explorerService.getTokenUrl(chain.string, tokenId) ?: return@combine null
        BlockExplorerLink(name = link.name, link = link.link)
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
        val assetId = token.value?.id ?: return@launch
        state.update { it.copy(isImporting = true) }
        val added = runCatchingCancellable {
            withContext(Dispatchers.IO) {
                addCustomToken(selectedChain.value, assetId)
            }
        }.onFailure { Log.e(TAG, "add custom token failed for ${assetId.toIdentifier()}", it) }
        state.update { it.copy(isImporting = false) }
        if (added.isSuccess) {
            onFinish()
        }
    }

    private companion object {
        const val TAG = "AddAsset"
    }

    private data class State(
        val isQrScan: Boolean = false,
        val isSelectChain: Boolean = false,
        val isImporting: Boolean = false,
    ) {
        fun toUIState(): AddAssetUIState {
            return AddAssetUIState(
                scene = when {
                    isQrScan -> AddAssetUIState.Scene.QrScanner
                    isSelectChain -> AddAssetUIState.Scene.SelectChain
                    else -> AddAssetUIState.Scene.Form
                },
                isLoading = isImporting,
            )
        }
    }
}

suspend fun searchToken(
    searchCustomToken: SearchCustomToken,
    observeToken: ObserveToken,
    assetId: AssetId,
): TokenSearchState {
    return try {
        searchCustomToken(assetId)
        val found = observeToken(assetId).firstOrNull() != null
        if (found) TokenSearchState.Idle else TokenSearchState.Error
    } catch (_: Exception) {
        TokenSearchState.Error
    }
}

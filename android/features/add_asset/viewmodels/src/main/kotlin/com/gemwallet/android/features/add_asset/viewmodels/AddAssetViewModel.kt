package com.gemwallet.android.features.add_asset.viewmodels

import android.util.Log
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.add_asset.viewmodels.models.AddAssetUIState
import com.gemwallet.android.features.add_asset.viewmodels.models.TokenSearchState
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemAddAssetServiceInterface
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AddAssetViewModel @Inject constructor(
    getSession: GetSession,
    private val service: GemAddAssetServiceInterface,
) : ViewModel() {

    private val state = MutableStateFlow(State())
    val uiState = state.map { it.toUIState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, AddAssetUIState())

    val chainFilter = TextFieldState()

    private val wallet = getSession().map { it?.wallet }.filterNotNull()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableChains = wallet.map { wallet ->
        wallet?.let { service.chains(it.toJson()).mapNotNull { chain -> chain.toChain() } }
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val chains = snapshotFlow { chainFilter.text }.combine(availableChains) { query, availableChains ->
        availableChains?.let { service.matchingChains(it.map { chain -> chain.string }, query.toString()).mapNotNull { chain -> chain.toChain() } } ?: emptyList()
    }
    .flowOn(Dispatchers.IO)
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val chain = MutableStateFlow<Chain?>(null)
    val selectedChain = availableChains.combine(chain) { availableChains, chain ->
        chain ?: service.defaultChain(availableChains.orEmpty().map { it.string })?.toChain() ?: Chain.Ethereum
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, Chain.Ethereum)

    val addressState = mutableStateOf("")

    val searchState = snapshotFlow { addressState.value }.combine(selectedChain) { address, chain -> chain to address }
        .flatMapLatest { (chain, address) ->
            flow {
                if (address.isEmpty()) {
                    emit(TokenSearchState.Idle)
                    return@flow
                }
                emit(TokenSearchState.Loading)
                emit(searchToken(chain, address))
            }
        }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, TokenSearchState.Idle)

    val token = searchState.map { (it as? TokenSearchState.Found)?.asset }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val buttonState = combine(token, uiState) { token, uiState ->
        buttonState(enabled = token != null, loading = uiState.isLoading)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Disabled)

    val explorerLink = token.map { token ->
        val tokenId = token?.id?.tokenId ?: return@map null
        val link = service.tokenUrl(token.id.chain.string, tokenId) ?: return@map null
        BlockExplorerLink(name = link.name, link = link.link)
    }
    .flowOn(Dispatchers.IO)
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
        val asset = token.value ?: return@launch
        val wallet = wallet.value ?: return@launch
        state.update { it.copy(isImporting = true) }
        val added = runCatchingCancellable {
            withContext(Dispatchers.IO) {
                service.add(wallet.toJson(), asset.id.toIdentifier())
            }
        }.onFailure { Log.e(TAG, "add custom token failed for ${asset.id.toIdentifier()}", it) }
        state.update { it.copy(isImporting = false) }
        if (added.isSuccess) {
            onFinish()
        }
    }

    private suspend fun searchToken(chain: Chain, address: String): TokenSearchState = try {
        TokenSearchState.Found(service.token(chain.string, address).toPrimitives())
    } catch (_: Exception) {
        TokenSearchState.Error
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

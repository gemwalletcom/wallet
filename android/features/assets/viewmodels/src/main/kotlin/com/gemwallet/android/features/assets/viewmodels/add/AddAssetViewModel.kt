package com.gemwallet.android.features.assets.viewmodels.add

import android.content.Context
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.assets.viewmodels.add.models.AddAssetUIState
import com.gemwallet.android.features.assets.viewmodels.add.models.verificationWarningListItem
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.ButtonState
import com.gemwallet.android.ui.models.buttonState
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
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
import uniffi.gemstone.GemAddAssetPhase
import uniffi.gemstone.GemAddAssetServiceInterface
import uniffi.gemstone.GemAddAssetSession
import uniffi.gemstone.GemChainServiceInterface
import uniffi.gemstone.GemListSection
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AddAssetViewModel @Inject constructor(
    getSession: GetSession,
    private val service: GemAddAssetServiceInterface,
    private val chainService: GemChainServiceInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val state = MutableStateFlow(State())
    val uiState = state.map { it.toUIState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, AddAssetUIState())

    val chainFilter = TextFieldState()

    private val wallet = getSession().map { it?.wallet }.filterNotNull()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val chainPicker = wallet.map { wallet -> wallet?.let { service.chainPicker(it.toGem()) } }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableChains = chainPicker.map { picker -> picker?.chains?.map { chain -> chain.requireChain() } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val showsChainPicker = chainPicker.map { it?.showsPicker == true }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val chains = snapshotFlow { chainFilter.text }.combine(availableChains) { query, availableChains ->
        availableChains?.let { chainService.chainRows(it.map { chain -> chain.string }, query.toString()) } ?: emptyList()
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val chain = MutableStateFlow<Chain?>(null)
    val selectedChain = chainPicker.combine(chain) { picker, chain ->
        chain ?: picker?.defaultChain?.requireChain()
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val addressState = mutableStateOf("")

    private val session = snapshotFlow { addressState.value }.combine(selectedChain) { address, chain -> chain to address }
        .flatMapLatest { (chain, address) ->
            flow {
                val input = service.newSession(chain?.string).onAddress(address)
                if (!input.searchesToken()) {
                    emit(input)
                    return@flow
                }
                emit(input.onLoading())
                emit(searchToken(input, requireNotNull(chain), address))
            }
        }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, service.newSession(null))

    val searchState = session.map { it.viewState().phase }
        .stateIn(viewModelScope, SharingStarted.Eagerly, GemAddAssetPhase.Idle)

    val isSearching: StateFlow<Boolean> = searchState.map { it is GemAddAssetPhase.Loading }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val token = session.map { it.asset?.toPrimitives() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val verificationWarningRow: StateFlow<ListItemModel?> = token.map { if (it == null) null else verificationWarningListItem(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val sections: StateFlow<List<GemListSection>> = session.map { service.sections(it) }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val buttonState = combine(session, uiState) { session, uiState ->
        session.onAdding(uiState.isLoading).viewState().button.buttonState()
    }.stateIn(viewModelScope, SharingStarted.Eagerly, ButtonState.Disabled)

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
            withContext(ioDispatcher) {
                service.add(wallet.toGem(), asset.id.toIdentifier())
            }
        }
        state.update { it.copy(isImporting = false, error = added.exceptionOrNull()?.errorText()?.text(context)) }
        if (added.isSuccess) {
            onFinish()
        }
    }

    fun clearError() = state.update { it.copy(error = null) }

    private suspend fun searchToken(session: GemAddAssetSession, chain: Chain, address: String): GemAddAssetSession = runCatchingCancellable { session.onFound(chain.string, address, service.token(chain.string, address)) }
        .getOrDefault(session.onFailed(chain.string, address))

    private data class State(val isQrScan: Boolean = false, val isSelectChain: Boolean = false, val isImporting: Boolean = false, val error: String? = null) {
        fun toUIState(): AddAssetUIState = AddAssetUIState(
            scene = when {
                isQrScan -> AddAssetUIState.Scene.QrScanner
                isSelectChain -> AddAssetUIState.Scene.SelectChain
                else -> AddAssetUIState.Scene.Form
            },
            isLoading = isImporting,
            error = error,
        )
    }
}

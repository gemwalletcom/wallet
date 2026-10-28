package com.gemwallet.android.features.onboarding.viewmodels.import_wallet

import android.content.Context
import androidx.annotation.StringRes
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.clearText
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.device.cases.EnablePushForNewWallet
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.ui.importWallet
import com.gemwallet.android.ui.localization.fieldStringRes
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.tabStringRes
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.models.name.NameRecordController
import com.wallet.core.primitives.WalletSource
import dagger.assisted.Assisted
import dagger.assisted.AssistedFactory
import dagger.assisted.AssistedInject
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import uniffi.gemstone.GemNameIndicator
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemWalletImportKind
import uniffi.gemstone.GemWalletImportResult
import uniffi.gemstone.GemWalletServiceInterface
import uniffi.gemstone.phraseSuggestions

@HiltViewModel(assistedFactory = ImportWalletViewModel.Factory::class)
class ImportWalletViewModel @AssistedInject constructor(
    @Assisted importType: ImportType,
    private val service: GemWalletServiceInterface,
    nameService: GemNameServiceInterface,
    private val enablePushForNewWallet: EnablePushForNewWallet,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    val inputState = TextFieldState()
    private val state = MutableStateFlow(ImportWalletUIState())
    private val input = snapshotFlow { inputState.text.toString() to inputState.selection.end }
    private val isImporting = MutableStateFlow(false)
    private val _isImported = MutableStateFlow(false)
    val isImported: StateFlow<Boolean> = _isImported.asStateFlow()
    val uiState = combine(state, isImporting) { state, importing -> state.copy(loading = importing) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ImportWalletUIState())
    val suggestions: StateFlow<List<String>> = combine(input, state) { (value, cursor), state ->
        if (cursor >= value.length && state.importType.kind.supportsPhraseSuggestions()) phraseSuggestions(value.lastWord()) else emptyList()
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val nameRecordController = NameRecordController(nameService, viewModelScope)
    val nameResolveState: StateFlow<GemNameRecordState> = nameRecordController.state
    val nameResolveIndicator: StateFlow<GemNameIndicator?> = nameResolveState.map { it.indicator() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    init {
        val screen = service.importScreen(importType.chain?.string)
        state.update {
            it.copy(
                importType = importType,
                title = screen.title.string(context),
                kinds = screen.kinds,
                showsTabs = screen.showsKinds,
            )
        }
        viewModelScope.launch {
            snapshotFlow { inputState.text.toString() }.collect(::onInput)
        }
    }

    fun importKind(type: ImportType) {
        nameRecordController.reset()
        resetInput()
        state.update {
            it.copy(
                importType = type,
                dataError = null,
            )
        }
    }

    private fun onInput(value: String) {
        clearError()
        val importType = state.value.importType
        if (importType.kind.resolvesNames()) {
            nameRecordController.getNameRecord(value, importType.chain)
        } else {
            nameRecordController.reset()
        }
    }

    fun selectSuggestion(word: String) {
        val value = inputState.text.toString()
        inputState.setTextAndPlaceCursorAtEnd(value.dropLast(value.lastWord().length) + word + " ")
    }

    private fun clearError() {
        state.update { it.copy(dataError = null) }
    }

    fun clearInput() {
        inputState.clearText()
    }

    private fun resetInput() {
        clearInput()
        isImporting.value = false
    }

    fun import() {
        if (isImporting.value) {
            return
        }
        val data = inputState.text.toString()
        // The text observer may not have seen the last edit yet, so the record could still be for the previous input.
        onInput(data)
        val nameRecord = nameRecordController.state.value.record()
        isImporting.value = true

        viewModelScope.launch(ioDispatcher) {
            try {
                val importType = state.value.importType
                val imported = service.importWallet(importType.kind, importType.chain, data, nameRecord, WalletSource.Import, context)
                isImporting.value = false
                when (imported) {
                    is GemWalletImportResult.New -> {
                        enablePushForNewWallet.enablePushForNewWallet()
                        _isImported.value = true
                    }

                    is GemWalletImportResult.Existing -> state.update { it.copy(existingWalletName = imported.wallet.name) }
                }
            } catch (err: CancellationException) {
                throw err
            } catch (err: Throwable) {
                state.update { it.copy(dataError = err.errorText().text(context)) }
                isImporting.value = false
            }
        }
    }

    fun onImportedHandled() {
        _isImported.value = false
    }

    fun dismissExistingWallet() {
        state.update { it.copy(existingWalletName = null) }
    }

    fun continueExistingWallet(onImported: () -> Unit) {
        dismissExistingWallet()
        enablePushForNewWallet.enablePushForNewWallet()
        onImported()
    }

    @AssistedFactory
    interface Factory {
        fun create(importType: ImportType): ImportWalletViewModel
    }
}

data class ImportWalletUIState(
    val loading: Boolean = false,
    val importType: ImportType = ImportType(GemWalletImportKind.PHRASE),
    val title: String = "",
    val kinds: List<GemWalletImportKind> = emptyList(),
    val showsTabs: Boolean = false,
    val dataError: String? = null,
    val existingWalletName: String? = null,
) {
    val tabs: List<ImportTabUIModel>
        get() = kinds.map { kind -> ImportTabUIModel(type = importType.copy(kind = kind), title = kind.tabStringRes(), isSelected = kind == importType.kind) }

    val input: ImportInputUIModel
        get() = importType.kind.inputUiModel()

    val showsScan: Boolean
        get() = importType.chain != null
}

@Immutable
data class ImportTabUIModel(val type: ImportType, @param:StringRes val title: Int, val isSelected: Boolean)

@Immutable
data class ImportInputUIModel(@param:StringRes val placeholder: Int, val protectsInput: Boolean, val supportsPhraseSuggestions: Boolean, val showsViewOnlyWarning: Boolean)

internal fun GemWalletImportKind.inputUiModel() = ImportInputUIModel(
    placeholder = fieldStringRes(),
    protectsInput = protectsInput(),
    supportsPhraseSuggestions = supportsPhraseSuggestions(),
    showsViewOnlyWarning = showsViewOnlyWarning(),
)

private fun String.lastWord(): String = takeLastWhile { !it.isWhitespace() }

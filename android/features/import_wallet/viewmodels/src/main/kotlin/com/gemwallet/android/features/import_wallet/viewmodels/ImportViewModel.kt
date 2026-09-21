package com.gemwallet.android.features.import_wallet.viewmodels

import android.content.Context
import androidx.annotation.StringRes
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.wallet_import.values.WalletImportResult
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ext.words
import com.gemwallet.android.features.import_wallet.viewmodels.localization.fieldStringRes
import com.gemwallet.android.features.import_wallet.viewmodels.localization.tabStringRes
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.ui.components.fields.NameResolveIndicatorUIModel
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.models.name.NameRecordController
import com.gemwallet.android.ui.style.indicator
import com.wallet.core.primitives.WalletSource
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.flow.updateAndGet
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemMnemonicInterface
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemWalletImportKind
import uniffi.gemstone.GemWalletImportResult
import uniffi.gemstone.GemWalletImportSession
import uniffi.gemstone.GemWalletServiceInterface
import javax.inject.Inject

@HiltViewModel
class ImportViewModel @Inject constructor(
    private val service: GemWalletServiceInterface,
    nameService: GemNameServiceInterface,
    private val mnemonic: GemMnemonicInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    fun invalidPhraseWords(text: String): Set<String> = mnemonic.findInvalidWords(text.words()).toSet()

    private val state = MutableStateFlow(ImportViewModelState())
    private val session = MutableStateFlow(GemWalletImportSession(GemWalletImportKind.PHRASE, "", null, false))
    val uiState = combine(state, session) { state, session -> state.toUIState(session.isImporting) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ImportUIState())
    val suggestions: StateFlow<List<String>> = session.map { it.suggestions() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val nameRecordController = NameRecordController(nameService, viewModelScope)
    val nameResolveState: StateFlow<GemNameRecordState> = nameRecordController.state
    val nameResolveIndicator: StateFlow<NameResolveIndicatorUIModel?> = nameResolveState.map { it.indicator() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun importKind(type: ImportType) {
        nameRecordController.reset()
        session.update { it.onKindChanged(type.kind) }
        state.update {
            it.copy(
                importType = type,
                dataError = null,
            )
        }
    }

    fun onInput(value: String, cursor: Int) {
        session.update { it.onInputChanged(value, cursor.toUInt()) }
        val importType = state.value.importType
        if (importType.kind.resolvesNames()) {
            nameRecordController.getNameRecord(value, importType.chain)
        } else {
            nameRecordController.reset()
        }
    }

    fun selectSuggestion(word: String): ImportTextUIModel {
        val next = session.updateAndGet { it.onSuggestionSelected(word) }
        return ImportTextUIModel(next.text, next.cursor?.toInt() ?: next.text.length)
    }

    fun clearInput() = session.update { it.onInputChanged("", null) }

    fun importSelect(importType: ImportType) = viewModelScope.launch {
        session.update { it.onKindChanged(importType.kind) }
        val defaultName = runCatchingCancellable {
            withContext(ioDispatcher) { service.defaultWalletName(importType.chain?.string) }
        }
        val screen = service.importScreen(importType.chain?.string)
        state.update {
            it.copy(
                importType = importType,
                defaultWalletName = defaultName.getOrNull()?.text?.string(context) ?: it.defaultWalletName,
                dataError = defaultName.exceptionOrNull() ?: it.dataError,
                title = screen.title.string(context),
                tabs = screen.kinds,
                showsTabs = screen.showsKinds,
            )
        }
    }

    fun import(generatedName: String, onImported: (WalletImportResult) -> Unit) {
        if (session.value.isImporting) {
            return
        }
        val nameRecord = nameRecordController.state.value.record()
        val data = session.updateAndGet { it.onImporting(true) }.text

        viewModelScope.launch(ioDispatcher) {
            try {
                val importType = state.value.importType
                val import = service.importRequest(importType.kind, importType.chain?.string, data, nameRecord)
                val walletName = service.importName(nameRecord, generatedName)
                val result = when (val imported = service.importWallet(walletName, import, WalletSource.Import.toGem())) {
                    is GemWalletImportResult.Existing -> WalletImportResult.Existing(imported.wallet.toPrimitives())
                    is GemWalletImportResult.New -> WalletImportResult.New(imported.wallet.toPrimitives())
                }
                service.setCurrentWalletId(result.wallet.id.id)
                state.update { it.copy(dataError = null) }
                session.update { it.onImporting(false) }
                withContext(Dispatchers.Main) {
                    when (result) {
                        is WalletImportResult.New -> onImported(result)
                        is WalletImportResult.Existing -> state.update { it.copy(existingWalletResult = result) }
                    }
                }
            } catch (err: CancellationException) {
                throw err
            } catch (err: Throwable) {
                state.update { it.copy(dataError = err) }
                session.update { it.onImporting(false) }
            }
        }
    }

    fun dismissExistingWallet() {
        state.update { it.copy(existingWalletResult = null) }
    }
}

data class ImportViewModelState(
    val error: String = "",
    val importType: ImportType = ImportType(GemWalletImportKind.PHRASE),
    val defaultWalletName: String? = null,
    val title: String = "",
    val tabs: List<GemWalletImportKind> = emptyList(),
    val showsTabs: Boolean = false,
    val dataError: Throwable? = null,
    val existingWalletResult: WalletImportResult.Existing? = null,
) {
    fun toUIState(loading: Boolean): ImportUIState = ImportUIState(
        loading = loading,
        error = error,
        defaultWalletName = defaultWalletName,
        title = title,
        showsTabs = showsTabs,
        tabs = tabs.map { kind -> ImportTabUIModel(type = importType.copy(kind = kind), title = kind.tabStringRes(), isSelected = kind == importType.kind) },
        input = importType.kind.inputUiModel(),
        importType = importType,
        dataError = dataError,
        existingWalletResult = existingWalletResult,
    )
}

data class ImportUIState(
    val loading: Boolean = false,
    val error: String = "",
    val importType: ImportType = ImportType(GemWalletImportKind.PHRASE),
    val defaultWalletName: String? = null,
    val title: String = "",
    val tabs: List<ImportTabUIModel> = emptyList(),
    val showsTabs: Boolean = false,
    val input: ImportInputUIModel = GemWalletImportKind.PHRASE.inputUiModel(),
    val dataError: Throwable? = null,
    val existingWalletResult: WalletImportResult.Existing? = null,
)

data class ImportTextUIModel(val text: String, val cursor: Int)

data class ImportTabUIModel(val type: ImportType, @StringRes val title: Int, val isSelected: Boolean)

data class ImportInputUIModel(@StringRes val placeholder: Int, val isPhrase: Boolean, val protectsInput: Boolean, val supportsPhraseSuggestions: Boolean, val showsViewOnlyWarning: Boolean)

internal fun GemWalletImportKind.inputUiModel() = ImportInputUIModel(
    placeholder = fieldStringRes(),
    isPhrase = when (this) {
        GemWalletImportKind.PHRASE -> true
        GemWalletImportKind.ADDRESS, GemWalletImportKind.PRIVATE_KEY -> false
    },
    protectsInput = protectsInput(),
    supportsPhraseSuggestions = supportsPhraseSuggestions(),
    showsViewOnlyWarning = showsViewOnlyWarning(),
)

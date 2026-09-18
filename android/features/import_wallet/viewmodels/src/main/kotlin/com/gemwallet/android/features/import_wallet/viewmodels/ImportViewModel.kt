package com.gemwallet.android.features.import_wallet.viewmodels

import android.content.Context
import androidx.annotation.StringRes
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_import.values.WalletImportResult
import com.gemwallet.android.data.services.gemstone.di.IoDispatcher
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
import javax.inject.Inject
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemMnemonicInterface
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemWalletImportKind
import uniffi.gemstone.GemWalletImportResult
import uniffi.gemstone.GemWalletServiceInterface

@HiltViewModel
class ImportViewModel @Inject constructor(
    private val service: GemWalletServiceInterface,
    nameService: GemNameServiceInterface,
    private val mnemonic: GemMnemonicInterface,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    fun invalidPhraseWords(text: String): Set<String> = mnemonic.findInvalidWords(text.words()).toSet()

    fun phraseSuggestions(word: String): List<String> = mnemonic.suggestWords(word, null)

    private val state = MutableStateFlow(ImportViewModelState())
    val uiState = state.map { it.toUIState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ImportUIState())

    private val nameRecordController = NameRecordController(nameService, viewModelScope)
    val nameResolveState: StateFlow<GemNameRecordState> = nameRecordController.state
    val nameResolveIndicator: StateFlow<NameResolveIndicatorUIModel?> = nameResolveState.map { it.indicator() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun importKind(type: ImportType) {
        nameRecordController.reset()
        state.update {
            it.copy(
                importType = type,
                dataError = null
            )
        }
    }

    fun onInput(value: String) {
        val importType = state.value.importType
        if (importType.kind.resolvesNames()) {
            nameRecordController.getNameRecord(value, importType.chain)
        } else {
            nameRecordController.reset()
        }
    }

    fun importSelect(importType: ImportType) = viewModelScope.launch {
        val defaultName = withContext(ioDispatcher) {
            service.defaultWalletName(importType.chain?.string)
        }
        val screen = service.importScreen(importType.chain?.string)
        state.update {
            it.copy(
                importType = importType,
                defaultWalletName = defaultName.text.string(context),
                title = screen.title.string(context),
                tabs = screen.kinds,
                showsTabs = screen.showsKinds,
            )
        }
    }

    fun import(
        generatedName: String,
        data: String,
        onImported: (WalletImportResult) -> Unit
    ) {
        if (state.value.loading) {
            return
        }
        val nameRecord = nameRecordController.state.value.record()
        state.update { it.copy(loading = true) }

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
                state.update { it.copy(dataError = null, loading = false) }
                withContext(Dispatchers.Main) {
                    when (result) {
                        is WalletImportResult.New -> onImported(result)
                        is WalletImportResult.Existing -> state.update { it.copy(existingWalletResult = result, loading = false) }
                    }
                }
            } catch (err: CancellationException) {
                throw err
            } catch (err: Throwable) {
                state.update { it.copy(dataError = err, loading = false) }
            }
        }
    }

    fun dismissExistingWallet() {
        state.update { it.copy(existingWalletResult = null) }
    }
}

data class ImportViewModelState(
    val loading: Boolean = false,
    val error: String = "",
    val importType: ImportType = ImportType(GemWalletImportKind.PHRASE),
    val defaultWalletName: String? = null,
    val title: String = "",
    val tabs: List<GemWalletImportKind> = emptyList(),
    val showsTabs: Boolean = false,
    val data: String = "",
    val dataError: Throwable? = null,
    val existingWalletResult: WalletImportResult.Existing? = null,
) {
    fun toUIState(): ImportUIState {
        return ImportUIState(
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

data class ImportTabUIModel(
    val type: ImportType,
    @StringRes val title: Int,
    val isSelected: Boolean,
)

data class ImportInputUIModel(
    @StringRes val placeholder: Int,
    val isPhrase: Boolean,
    val protectsInput: Boolean,
    val supportsPhraseSuggestions: Boolean,
    val showsViewOnlyWarning: Boolean,
)

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


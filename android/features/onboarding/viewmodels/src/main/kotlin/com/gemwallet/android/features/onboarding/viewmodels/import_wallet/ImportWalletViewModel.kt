package com.gemwallet.android.features.onboarding.viewmodels.import_wallet

import android.content.Context
import androidx.annotation.StringRes
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.device.cases.EnablePushForNewWallet
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.ui.components.fields.NameResolveIndicatorUIModel
import com.gemwallet.android.ui.importWallet
import com.gemwallet.android.ui.localization.fieldStringRes
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.tabStringRes
import com.gemwallet.android.ui.localization.text
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
import uniffi.gemstone.GemNameRecordState
import uniffi.gemstone.GemNameServiceInterface
import uniffi.gemstone.GemWalletImportKind
import uniffi.gemstone.GemWalletImportResult
import uniffi.gemstone.GemWalletServiceInterface
import uniffi.gemstone.phraseSuggestions
import javax.inject.Inject

@HiltViewModel
class ImportWalletViewModel @Inject constructor(
    private val service: GemWalletServiceInterface,
    nameService: GemNameServiceInterface,
    private val enablePushForNewWallet: EnablePushForNewWallet,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val state = MutableStateFlow(ImportWalletUIState())
    private val input = MutableStateFlow("")
    private val isTypingLastWord = MutableStateFlow(true)
    private val isImporting = MutableStateFlow(false)
    val uiState = combine(state, isImporting) { state, importing -> state.copy(loading = importing) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ImportWalletUIState())
    val suggestions: StateFlow<List<String>> = combine(input, isTypingLastWord, state) { input, isTypingLastWord, state ->
        if (isTypingLastWord && state.importType.kind.supportsPhraseSuggestions()) phraseSuggestions(input.lastWord()) else emptyList()
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val nameRecordController = NameRecordController(nameService, viewModelScope)
    val nameResolveState: StateFlow<GemNameRecordState> = nameRecordController.state
    val nameResolveIndicator: StateFlow<NameResolveIndicatorUIModel?> = nameResolveState.map { it.indicator() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

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

    fun onInput(value: String, cursor: Int) {
        input.value = value
        isTypingLastWord.value = cursor >= value.length
        val importType = state.value.importType
        if (importType.kind.resolvesNames()) {
            nameRecordController.getNameRecord(value, importType.chain)
        } else {
            nameRecordController.reset()
        }
    }

    fun selectSuggestion(word: String): String {
        isTypingLastWord.value = true
        return input.updateAndGet { it.dropLast(it.lastWord().length) + word + " " }
    }

    fun clearInput() {
        input.value = ""
        isTypingLastWord.value = true
    }

    private fun resetInput() {
        clearInput()
        isImporting.value = false
    }

    fun importSelect(importType: ImportType) {
        resetInput()
        val screen = service.importScreen(importType.chain?.string)
        state.update {
            it.copy(
                importType = importType,
                title = screen.title.string(context),
                kinds = screen.kinds,
                showsTabs = screen.showsKinds,
            )
        }
    }

    fun import(onImported: () -> Unit) {
        if (isImporting.value) {
            return
        }
        val nameRecord = nameRecordController.state.value.record()
        isImporting.value = true
        val data = input.value

        viewModelScope.launch(ioDispatcher) {
            try {
                val importType = state.value.importType
                val imported = service.importWallet(importType.kind, importType.chain, data, nameRecord, WalletSource.Import, context)
                state.update { it.copy(dataError = null) }
                isImporting.value = false
                withContext(Dispatchers.Main) {
                    when (imported) {
                        is GemWalletImportResult.New -> {
                            enablePushForNewWallet.enablePushForNewWallet()
                            onImported()
                        }

                        is GemWalletImportResult.Existing -> state.update { it.copy(existingWalletName = imported.wallet.name) }
                    }
                }
            } catch (err: CancellationException) {
                throw err
            } catch (err: Throwable) {
                state.update { it.copy(dataError = err.errorText().text(context)) }
                isImporting.value = false
            }
        }
    }

    fun dismissExistingWallet() {
        state.update { it.copy(existingWalletName = null) }
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
}

data class ImportTabUIModel(val type: ImportType, @StringRes val title: Int, val isSelected: Boolean)

data class ImportInputUIModel(@StringRes val placeholder: Int, val protectsInput: Boolean, val supportsPhraseSuggestions: Boolean, val showsViewOnlyWarning: Boolean)

internal fun GemWalletImportKind.inputUiModel() = ImportInputUIModel(
    placeholder = fieldStringRes(),
    protectsInput = protectsInput(),
    supportsPhraseSuggestions = supportsPhraseSuggestions(),
    showsViewOnlyWarning = showsViewOnlyWarning(),
)

private fun String.lastWord(): String = takeLastWhile { !it.isWhitespace() }

package com.gemwallet.android.features.import_wallet.viewmodels

import com.gemwallet.android.ext.toPrimitives
import uniffi.gemstone.GemWalletDefaultName
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet_import.values.WalletImportResult
import kotlinx.coroutines.CancellationException
import uniffi.gemstone.GemNameServiceInterface
import com.gemwallet.android.ext.words
import uniffi.gemstone.GemMnemonicInterface
import uniffi.gemstone.GemWalletServiceInterface
import uniffi.gemstone.GemWalletImportResult
import com.gemwallet.android.ext.toGem
import com.wallet.core.primitives.WalletSource
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.model.toWalletType
import com.gemwallet.android.ui.models.name.NameRecordState
import com.gemwallet.android.ui.models.name.NameRecordController
import com.wallet.core.primitives.WalletType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject

@HiltViewModel
class ImportViewModel @Inject constructor(
    private val service: GemWalletServiceInterface,
    nameService: GemNameServiceInterface,
    private val mnemonic: GemMnemonicInterface,
) : ViewModel() {

    fun invalidPhraseWords(text: String): Set<String> = mnemonic.findInvalidWords(text.words()).toSet()

    fun phraseSuggestions(word: String): List<String> = mnemonic.suggestWords(word, null)

    private val state = MutableStateFlow(ImportViewModelState())
    val uiState = state.map { it.toUIState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ImportUIState())

    private val nameRecordController = NameRecordController(nameService, viewModelScope)
    val nameResolveState: StateFlow<NameRecordState> = nameRecordController.state

    fun chainType(walletType: WalletType) {
        nameRecordController.reset()
        state.update {
            it.copy(
                importType = it.importType.copy(walletType = walletType),
                dataError = null
            )
        }
    }

    fun onInput(value: String) {
        val importType = state.value.importType
        when (importType.walletType) {
            WalletType.View -> nameRecordController.getNameRecord(value, importType.chain)
            else -> nameRecordController.reset()
        }
    }

    fun importSelect(importType: ImportType) = viewModelScope.launch {
        val defaultName = withContext(Dispatchers.IO) {
            service.defaultWalletName(importType.chain?.string)
        }
        val chainName = if (importType.walletType == WalletType.Multicoin) "" else importType.chain?.networkName().orEmpty()
        val tabs = service.importKinds(importType.chain?.string).map { it.toWalletType(importType.chain) }
        state.update {
            it.copy(
                importType = importType,
                defaultWalletName = defaultName.name,
                chainName = chainName,
                tabs = tabs,
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
        val nameRecord = nameRecordController.state.value.nameRecord
        state.update { it.copy(loading = true) }

        viewModelScope.launch(Dispatchers.IO) {
            try {
                val importType = state.value.importType
                val import = service.importRequest(importType.kind, importType.chain?.string, data, nameRecord?.toGem())
                val walletName = service.importName(nameRecord?.toGem(), generatedName)
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
    val importType: ImportType = ImportType(WalletType.Multicoin),
    val defaultWalletName: String = "",
    val chainName: String = "",
    val tabs: List<WalletType> = emptyList(),
    val data: String = "",
    val dataError: Throwable? = null,
    val existingWalletResult: WalletImportResult.Existing? = null,
) {
    fun toUIState(): ImportUIState {
        return ImportUIState(
            loading = loading,
            error = error,
            defaultWalletName = defaultWalletName,
            chainName = chainName,
            tabs = tabs,
            importType = importType,
            dataError = dataError,
            existingWalletResult = existingWalletResult,
        )
    }
}

data class ImportUIState(
    val loading: Boolean = false,
    val error: String = "",
    val importType: ImportType = ImportType(WalletType.Multicoin),
    val defaultWalletName: String = "",
    val chainName: String = "",
    val tabs: List<WalletType> = emptyList(),
    val dataError: Throwable? = null,
    val existingWalletResult: WalletImportResult.Existing? = null,
)


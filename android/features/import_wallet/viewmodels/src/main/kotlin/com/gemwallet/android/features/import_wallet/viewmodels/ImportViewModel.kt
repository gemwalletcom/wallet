package com.gemwallet.android.features.import_wallet.viewmodels

import com.gemwallet.android.blockchain.operators.InvalidWords
import com.gemwallet.android.blockchain.operators.ValidatePhraseOperator
import com.gemwallet.android.blockchain.operators.gemstone.GemFindPhraseWord
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.gemwallet.android.application.wallet_import.values.ImportError
import com.gemwallet.android.application.wallet_import.cases.ImportWalletService
import com.gemwallet.android.application.wallet_import.values.WalletImportResult
import com.gemwallet.android.application.recipient.cases.GetNameRecord
import uniffi.gemstone.GemWalletService
import com.gemwallet.android.ext.networkName
import com.gemwallet.android.model.ImportType
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
    private val walletService: GemWalletService,
    private val importWalletService: ImportWalletService,
    private val setCurrentWallet: SetCurrentWallet,
    private val validatePhrase: ValidatePhraseOperator,
    private val findPhraseWord: GemFindPhraseWord,
    getNameRecord: GetNameRecord,
) : ViewModel() {

    fun invalidPhraseWords(text: String): Set<String> =
        (validatePhrase(text).exceptionOrNull() as? InvalidWords)
            ?.words
            .orEmpty()
            .filter { it.isNotBlank() }
            .toSet()

    fun phraseSuggestions(word: String): List<String> = findPhraseWord(word)

    private val state = MutableStateFlow(ImportViewModelState())
    val uiState = state.map { it.toUIState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, ImportUIState())

    private val nameRecordController = NameRecordController(getNameRecord, viewModelScope)
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
        val generatedNameIndex = walletService.nextWalletIndex()
        val chainName = if (importType.walletType == WalletType.Multicoin) "" else importType.chain?.networkName().orEmpty()
        state.update {
            it.copy(
                importType = importType,
                generatedNameIndex = generatedNameIndex,
                chainName = chainName,
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
                val result = importWalletService.importWallet(
                    importType = state.value.importType,
                    walletName = nameRecord?.name?.takeIf { it.isNotBlank() } ?: generatedName,
                    data = if (nameRecord?.address.isNullOrEmpty()) data.trim() else nameRecord.address,
                )
                state.update { it.copy(dataError = null, loading = false) }
                withContext(Dispatchers.Main) {
                    when (result) {
                        is WalletImportResult.New -> onImported(result)
                        is WalletImportResult.Existing -> {
                            setCurrentWallet.setCurrentWallet(result.wallet.id)
                            state.update {
                                it.copy(existingWalletResult = result, loading = false)
                            }
                        }
                    }
                }
            } catch (err: Throwable) {
                state.update { it.copy(dataError = (err as? ImportError) ?: ImportError.CreateError(err.message.orEmpty()), loading = false) }
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
    val generatedNameIndex: Int = 0,
    val chainName: String = "",
    val data: String = "",
    val dataError: ImportError? = null,
    val existingWalletResult: WalletImportResult.Existing? = null,
) {
    fun toUIState(): ImportUIState {
        return ImportUIState(
            loading = loading,
            error = error,
            generatedNameIndex = generatedNameIndex,
            chainName = chainName,
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
    val generatedNameIndex: Int = 0,
    val chainName: String = "",
    val dataError: ImportError? = null,
    val existingWalletResult: WalletImportResult.Existing? = null,
)

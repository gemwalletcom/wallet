package com.gemwallet.android.features.create_wallet.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ui.components.screen.PhraseRow
import com.gemwallet.android.ui.components.screen.phraseRows
import com.gemwallet.android.ui.importWallet
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.WalletSource
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemCopy
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemVerifyPhraseSession
import uniffi.gemstone.GemVerifyPhraseViewState
import uniffi.gemstone.GemWalletImportKind
import uniffi.gemstone.GemWalletServiceInterface
import uniffi.gemstone.secretPhraseCopy
import javax.inject.Inject

@HiltViewModel
class CreateWalletViewModel @Inject constructor(private val service: GemWalletServiceInterface, @param:ApplicationContext private val context: Context, @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher) : ViewModel() {

    private val state = MutableStateFlow(CreateWalletViewModelState())
    val uiState = state.asStateFlow()

    val errorText: StateFlow<String?> = state.map { it.dataError?.text(context) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val verification = MutableStateFlow<GemVerifyPhraseSession?>(null)

    val verificationState: StateFlow<GemVerifyPhraseViewState?> = verification.map { it?.viewState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val phraseRows: StateFlow<List<PhraseRow>> = state.map { phraseRows(it.data) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val verifiedRows: StateFlow<List<PhraseRow>> = verificationState.map { phraseRows(it?.verified.orEmpty()) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun phraseCopy(): GemCopy = secretPhraseCopy(state.value.data)

    fun onPickWord(choice: Int): Boolean {
        val current = verification.value ?: return false
        val next = current.onPick(choice.toUInt())
        verification.value = next
        return next != current
    }

    init {
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { service.createWallet() }
                .onSuccess { words -> state.update { it.copy(data = words) } }
                .onFailure { err -> state.update { it.copy(dataError = err.errorText()) } }
        }
    }

    fun dismissSafeMessage() {
        state.update {
            it.copy(isShowSafeMessage = false)
        }
    }

    fun confirmPhrase() {
        verification.value = service.verifyPhraseSession(state.value.data)
        state.update { it.copy(isShowSafeMessage = true) }
    }

    fun createWallet(onCreated: () -> Unit) {
        val current = verification.value ?: return
        if (current.isCreating) {
            return
        }
        verification.value = current.onCreating(true)
        state.update { it.copy(isShowSafeMessage = true) }
        viewModelScope.launch(ioDispatcher) {
            try {
                service.importWallet(GemWalletImportKind.PHRASE, null, state.value.data.joinToString(" "), null, WalletSource.Create, context)
                withContext(Dispatchers.Main) { onCreated() }
            } catch (err: CancellationException) {
                throw err
            } catch (err: Throwable) {
                verification.update { it?.onCreating(false) }
                state.update { it.copy(dataError = err.errorText()) }
            }
        }
    }
}

data class CreateWalletViewModelState(val data: List<String> = emptyList(), val dataError: GemErrorText? = null, val isShowSafeMessage: Boolean = false) {
    override fun toString() = "CreateWalletViewModelState(wordCount=${data.size}, isShowSafeMessage=$isShowSafeMessage)"
}

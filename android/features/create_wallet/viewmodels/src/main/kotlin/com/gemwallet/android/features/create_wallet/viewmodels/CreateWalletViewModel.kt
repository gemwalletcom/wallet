package com.gemwallet.android.features.create_wallet.viewmodels

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.localization.string
import com.gemwallet.android.ui.localization.text
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
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
import uniffi.gemstone.GemErrorText
import uniffi.gemstone.GemVerifyPhraseSession
import uniffi.gemstone.GemVerifyPhraseViewState
import uniffi.gemstone.GemWalletDefaultName
import uniffi.gemstone.GemWalletImportKind
import uniffi.gemstone.GemWalletImportResult
import uniffi.gemstone.GemWalletServiceInterface
import javax.inject.Inject

@HiltViewModel
class CreateWalletViewModel @Inject constructor(private val service: GemWalletServiceInterface, @param:ApplicationContext private val context: Context, @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher) : ViewModel() {

    private val state = MutableStateFlow(CreateWalletViewModelState())
    val uiState = state.asStateFlow()

    val defaultNameText: StateFlow<String> = state.map { it.defaultName?.text?.string(context).orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val errorText: StateFlow<String?> = state.map { it.dataError?.text(context)?.ifBlank { context.getString(R.string.errors_unknown_try_again) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val verification = MutableStateFlow<GemVerifyPhraseSession?>(null)

    val verificationState: StateFlow<GemVerifyPhraseViewState?> = verification.map { it?.viewState() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun onPickWord(choice: Int): Boolean {
        val current = verification.value ?: return false
        val next = current.onPick(choice.toUInt())
        verification.value = next
        return next != current
    }

    init {
        viewModelScope.launch(ioDispatcher) {
            runCatchingCancellable { service.defaultWalletName(null) to service.createWallet() }
                .onSuccess { (defaultName, words) -> state.update { it.copy(defaultName = defaultName, data = words) } }
                .onFailure { err -> state.update { it.copy(dataError = err.errorText()) } }
        }
    }

    fun dismissSafeMessage() {
        state.update {
            it.copy(isShowSafeMessage = false)
        }
    }

    fun confirmPhrase(walletName: String) {
        verification.value = service.verifyPhraseSession(state.value.data)
        state.update {
            it.copy(
                name = walletName.ifEmpty { it.name },
                isShowSafeMessage = true,
            )
        }
    }

    fun createWallet(onCreated: (walletId: WalletId?) -> Unit) {
        if (state.value.loading) {
            return
        }
        state.update { it.copy(isShowSafeMessage = true, loading = true) }
        viewModelScope.launch(ioDispatcher) {
            val newState = try {
                val wallet = createWallet(state.value.name, state.value.data.joinToString(" "))
                withContext(Dispatchers.Main) {
                    onCreated(if (state.value.isExistingWallets()) wallet.id else null)
                }
                state.value.copy(loading = false)
            } catch (err: CancellationException) {
                throw err
            } catch (err: Throwable) {
                state.value.copy(loading = false, dataError = err.errorText())
            }
            state.update { newState }
        }
    }

    private suspend fun createWallet(name: String, phrase: String): Wallet {
        val wallet = when (val result = service.importWallet(name, service.importRequest(GemWalletImportKind.PHRASE, null, phrase, null), WalletSource.Create.toGem())) {
            is GemWalletImportResult.Existing -> result.wallet.toPrimitives()
            is GemWalletImportResult.New -> result.wallet.toPrimitives()
        }
        service.setCurrentWalletId(wallet.id.id)
        return wallet
    }
}

data class CreateWalletViewModelState(
    val loading: Boolean = false,
    val defaultName: GemWalletDefaultName? = null,
    val name: String = "",
    val data: List<String> = emptyList(),
    val dataError: GemErrorText? = null,
    val isShowSafeMessage: Boolean = false,
) {
    fun isExistingWallets() = defaultName?.hasExistingWallets == true

    override fun toString() = "CreateWalletViewModelState(loading=$loading, wordCount=${data.size}, isShowSafeMessage=$isShowSafeMessage)"
}

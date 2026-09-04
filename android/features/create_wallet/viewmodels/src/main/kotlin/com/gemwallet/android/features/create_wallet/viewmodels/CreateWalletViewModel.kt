package com.gemwallet.android.features.create_wallet.viewmodels

import uniffi.gemstone.GemWalletDefaultName
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.domains.wallet_import.multicoinImport
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import kotlinx.coroutines.CancellationException
import uniffi.gemstone.GemWalletServiceInterface
import uniffi.gemstone.GemWalletImportResult
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject

@HiltViewModel
class CreateWalletViewModel @Inject constructor(
    private val service: GemWalletServiceInterface,
) : ViewModel() {

    private val state = MutableStateFlow(CreateWalletViewModelState())
    val uiState = state.asStateFlow()

    init {
        viewModelScope.launch(Dispatchers.IO) {
            state.update { it.copy(defaultName = service.defaultWalletName(null)) }
            runCatchingCancellable { service.createWallet() }
                .onSuccess { words -> state.update { it.copy(data = words) } }
                .onFailure { err -> state.update { it.copy(dataError = err.message.orEmpty()) } }
        }
    }

    fun handleCreateDismiss() {
        state.update {
            it.copy(isShowSafeMessage = false)
        }
    }

    fun handleReadyToCreate(walletName: String) {
        state.update {
            it.copy(
                name = walletName.ifEmpty { it.name },
                isShowSafeMessage = true,
            )
        }
    }

    fun handleCreate(onCreated: (walletId: WalletId?) -> Unit) {
        if (state.value.loading) {
            return
        }
        state.update { it.copy(isShowSafeMessage = true, loading = true) }
        viewModelScope.launch(Dispatchers.IO) {
            val newState = try {
                val wallet = createWallet(state.value.name, state.value.data.joinToString(" "))
                withContext(Dispatchers.Main) {
                    onCreated(if (state.value.isExistingWallets()) wallet.id else null)
                }
                state.value.copy(loading = false)
            } catch (err: CancellationException) {
                throw err
            } catch (err: Throwable) {
                state.value.copy(loading = false, dataError = err.message.orEmpty())
            }
            state.update { newState }
        }
    }

    private suspend fun createWallet(name: String, phrase: String): Wallet {
        val wallet = when (val result = service.importWallet(name, multicoinImport(phrase).validated(), WalletSource.Create.toGem())) {
            is GemWalletImportResult.Existing -> result.wallet.decodeJson<Wallet>()
            is GemWalletImportResult.New -> result.wallet.decodeJson<Wallet>()
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
    val dataError: String? = null,
    val isShowSafeMessage: Boolean = false,
) {
    fun isExistingWallets() = defaultName.index() > 1
}

private fun GemWalletDefaultName?.index(): Int = when (this) {
    is GemWalletDefaultName.Multicoin -> index
    is GemWalletDefaultName.Chain -> index
    null -> 0
}

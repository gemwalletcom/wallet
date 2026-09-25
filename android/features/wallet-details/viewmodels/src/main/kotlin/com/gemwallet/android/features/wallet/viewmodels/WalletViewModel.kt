package com.gemwallet.android.features.wallet.viewmodels

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.data.services.store.queries.WalletQuery
import com.gemwallet.android.domains.wallet.WalletSecretInput
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.wallet.viewmodels.models.WalletDetailsUIModel
import com.gemwallet.android.features.wallet.viewmodels.models.WalletSecretUIModel
import com.gemwallet.android.features.wallet.viewmodels.models.uiModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.stringRes
import com.gemwallet.android.ui.localization.text
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemWalletDeletion
import uniffi.gemstone.GemWalletServiceInterface
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class WalletViewModel @Inject constructor(
    walletQuery: WalletQuery,
    private val service: GemWalletServiceInterface,
    savedStateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val walletId = savedStateHandle.requireWalletId()

    private val wallet = walletQuery(walletId)
        .mapLatest { wallet -> wallet?.let { service.walletDetails(it.toGem()) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val details: StateFlow<WalletDetailsUIModel?> = wallet.map { it?.uiModel() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val secret: StateFlow<WalletSecretUIModel?> = wallet.map { details ->
        details?.secretKind?.let { kind ->
            WalletSecretUIModel(WalletSecretInput(walletId, kind), ListItemModel(title = context.getString(R.string.common_show, context.getString(kind.stringRes()))))
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    fun setWalletName(name: String) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { service.rename(walletId.id, name) }
            .onFailure(::showError)
    }

    fun delete(onBoard: () -> Unit, onComplete: () -> Unit) = viewModelScope.launch {
        runCatchingCancellable {
            when (withContext(ioDispatcher) { service.deleteWallet(walletId.id) }) {
                GemWalletDeletion.WALLETS_REMAINING -> onComplete()
                GemWalletDeletion.LAST_WALLET_DELETED -> onBoard()
            }
        }
            .onFailure(::showError)
    }

    fun clearError() = errorState.update { null }

    private fun showError(error: Throwable) {
        errorState.value = error.errorText().text(context)
    }
}

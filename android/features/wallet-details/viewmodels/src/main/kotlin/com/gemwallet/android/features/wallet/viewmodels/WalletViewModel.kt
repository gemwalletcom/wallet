package com.gemwallet.android.features.wallet.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet.cases.DeleteWallet
import com.gemwallet.android.application.wallet.cases.GetWalletDetails
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.features.wallet.viewmodels.models.WalletSecretUIModel
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.localization.stringRes
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import javax.inject.Inject
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemWalletServiceInterface

@HiltViewModel
class WalletViewModel @Inject constructor(
    private val getWalletDetails: GetWalletDetails,
    private val service: GemWalletServiceInterface,
    private val deleteWallet: DeleteWallet,
    savedStateHandle: SavedStateHandle,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val walletId = savedStateHandle.requireWalletId()

    val wallet = getWalletDetails.getWallet(walletId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val secret: StateFlow<WalletSecretUIModel?> = wallet.map { details ->
        details?.secretKind?.let { kind ->
            WalletSecretUIModel(kind, ListItemModel(title = context.getString(R.string.common_show, context.getString(kind.stringRes()))))
        }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun setWalletName(name: String) = viewModelScope.launch(Dispatchers.IO) {
        runCatchingCancellable { service.rename(walletId.id, name) }
            .onFailure { Log.e(TAG, "renaming wallet ${walletId.id} failed", it) }
    }

    fun delete(onBoard: () -> Unit, onComplete: () -> Unit) = viewModelScope.launch(Dispatchers.IO) {
        deleteWallet.deleteWallet(walletId, onBoard, onComplete)
    }

    private companion object {
        const val TAG = "Wallet"
    }
}

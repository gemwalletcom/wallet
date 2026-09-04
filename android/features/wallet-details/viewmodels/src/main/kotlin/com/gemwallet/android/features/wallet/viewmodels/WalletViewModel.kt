package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet.cases.DeleteWallet
import com.gemwallet.android.application.wallet.cases.GetWalletDetails
import com.gemwallet.android.ext.runCatchingCancellable
import dagger.hilt.android.lifecycle.HiltViewModel
import uniffi.gemstone.GemWalletServiceInterface
import android.util.Log
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class WalletViewModel @Inject constructor(
    private val getWalletDetails: GetWalletDetails,
    private val service: GemWalletServiceInterface,
    private val deleteWallet: DeleteWallet,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val walletId = savedStateHandle.requireWalletId()

    val wallet = getWalletDetails.getWallet(walletId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

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

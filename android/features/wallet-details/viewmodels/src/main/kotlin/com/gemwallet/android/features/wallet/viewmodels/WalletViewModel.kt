package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet.coordinators.DeleteWallet
import com.gemwallet.android.application.wallet.coordinators.GetWalletDetails
import com.gemwallet.android.application.wallet.coordinators.SetWalletName
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class WalletViewModel @Inject constructor(
    private val getWalletDetails: GetWalletDetails,
    private val setWalletName: SetWalletName,
    private val deleteWallet: DeleteWallet,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val walletId = savedStateHandle.requireWalletId()

    val wallet = getWalletDetails.getWallet(walletId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun setWalletName(name: String) = viewModelScope.launch(Dispatchers.IO) {
        setWalletName.setWalletName(walletId, name)
    }

    fun delete(onBoard: () -> Unit, onComplete: () -> Unit) = viewModelScope.launch(Dispatchers.IO) {
        deleteWallet.deleteWallet(walletId, onBoard, onComplete)
    }
}

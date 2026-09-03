package com.gemwallet.android.features.wallets.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet.cases.DeleteWallet
import com.gemwallet.android.application.wallet.cases.GetAllWallets
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.wallet.core.primitives.WalletId
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemWalletServiceInterface
import javax.inject.Inject

@HiltViewModel
class WalletsViewModel @Inject constructor(
    private val getAllWallets: GetAllWallets,
    private val setCurrentWallet: SetCurrentWallet,
    private val service: GemWalletServiceInterface,
    private val deleteWallet: DeleteWallet,
) : ViewModel() {

    val wallets = getAllWallets.getAllWallets()
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun selectWallet(walletId: WalletId) = viewModelScope.launch(Dispatchers.IO) {
        setCurrentWallet.setCurrentWallet(walletId)
    }

    fun deleteWallet(walletId: WalletId, onBoard: () -> Unit) = viewModelScope.launch {
        deleteWallet.deleteWallet(walletId, onBoard) {}
    }

    fun togglePin(walletId: WalletId) = viewModelScope.launch(Dispatchers.IO) {
        val wallet = wallets.value.firstOrNull { it.id == walletId.id } ?: return@launch
        service.setPinned(walletId.id, !wallet.isPinned)
    }
}

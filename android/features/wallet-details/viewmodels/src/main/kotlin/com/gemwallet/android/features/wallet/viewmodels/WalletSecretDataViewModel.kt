package com.gemwallet.android.features.wallet.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.wallet.coordinators.GetWalletSecretData
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@HiltViewModel
class WalletSecretDataViewModel @Inject constructor(
    private val getWalletSecretData: GetWalletSecretData,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {
    private val walletId = savedStateHandle.requireWalletId()
    val walletType = MutableStateFlow(savedStateHandle.requireWalletType())

    val data = getWalletSecretData.getSecretData(walletId)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)
}

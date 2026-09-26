package com.gemwallet.android.features.onboarding.viewmodels.import_wallet

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemChainServiceInterface
import uniffi.gemstone.GemImportWalletTypes
import javax.inject.Inject

@HiltViewModel
class ImportWalletTypeViewModel @Inject constructor(private val chainService: GemChainServiceInterface) : ViewModel() {
    val chainFilter = TextFieldState()
    val types: StateFlow<GemImportWalletTypes> = snapshotFlow { chainFilter.text.toString() }
        .map { query -> chainService.importWalletTypes(query) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, chainService.importWalletTypes(""))
}

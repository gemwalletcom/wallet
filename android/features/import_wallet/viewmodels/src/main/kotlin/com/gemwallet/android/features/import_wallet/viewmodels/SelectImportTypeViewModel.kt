package com.gemwallet.android.features.import_wallet.viewmodels

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.ext.requireChain
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemChainServiceInterface
import javax.inject.Inject

@HiltViewModel
class SelectImportTypeViewModel @Inject constructor(private val chainService: GemChainServiceInterface) : ViewModel() {
    val chainFilter = TextFieldState()
    val chains: StateFlow<List<Chain>> = snapshotFlow { chainFilter.text.toString() }
        .map { query -> chainService.getChains(query).map { it.requireChain() } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
}

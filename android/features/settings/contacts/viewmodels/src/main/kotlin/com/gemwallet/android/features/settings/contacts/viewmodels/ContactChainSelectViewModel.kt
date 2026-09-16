package com.gemwallet.android.features.settings.contacts.viewmodels

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.ext.requireChain
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import uniffi.gemstone.GemChainServiceInterface
import javax.inject.Inject

@HiltViewModel
class ContactChainSelectViewModel @Inject constructor(
    private val chainService: GemChainServiceInterface,
) : ViewModel() {
    private val state = MutableStateFlow<List<Chain>>(emptyList())
    val chains = state.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())
    val chainFilter = TextFieldState()

    init {
        viewModelScope.launch {
            snapshotFlow { chainFilter.text }.collectLatest { query ->
                state.value = chainService.getChains(query.toString()).map { it.requireChain() }
            }
        }
    }
}

package com.gemwallet.android.features.recipient.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.cases.name.ResolveName
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.NameRecord
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.StateFlow
import javax.inject.Inject

@HiltViewModel
class AddressChainViewModel @Inject constructor(
    resolveName: ResolveName,
) : ViewModel() {

    private val resolver = NameResolveController(resolveName, viewModelScope)

    val uiState: StateFlow<NameRecordState> = resolver.state

    fun onNameRecord(chain: Chain?, nameRecord: String) = resolver.onNameRecord(chain, nameRecord)

    fun onInput(input: String, chain: Chain?) = resolver.onInput(input, chain)

    fun onResolved(onResolved: (NameRecord?) -> Unit) {
        resolver.onResolved = onResolved
    }
}

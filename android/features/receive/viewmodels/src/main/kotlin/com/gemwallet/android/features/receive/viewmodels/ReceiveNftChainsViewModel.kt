package com.gemwallet.android.features.receive.viewmodels

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import uniffi.gemstone.GemNftServiceInterface
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@HiltViewModel
class ReceiveNftChainsViewModel @Inject constructor(
    private val service: GemNftServiceInterface,
) : ViewModel() {

    val chainFilter = TextFieldState()

    private val accounts = snapshotFlow { chainFilter.text.toString() }
        .map { query -> service.receiveAccounts(query).mapNotNull { it.toPrimitives() } }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val chains = accounts.map { accounts -> accounts.map(Account::chain) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun addressFor(chain: Chain): String = accounts.value.firstOrNull { it.chain == chain }?.address.orEmpty()
}

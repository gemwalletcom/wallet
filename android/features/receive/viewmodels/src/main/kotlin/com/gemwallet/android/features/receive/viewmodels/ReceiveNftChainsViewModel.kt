package com.gemwallet.android.features.receive.viewmodels

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.serializer.toJson
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import uniffi.gemstone.GemNftServiceInterface
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@HiltViewModel
class ReceiveNftChainsViewModel @Inject constructor(
    getSession: GetSession,
    private val service: GemNftServiceInterface,
) : ViewModel() {

    val chainFilter = TextFieldState()

    private val session = getSession()

    private val accounts = combine(snapshotFlow { chainFilter.text.toString() }, session) { query, session ->
        session?.wallet?.let { service.receiveAccounts(it.accounts.map { account -> account.toGem() }, query).mapNotNull { account -> account.toPrimitives() } } ?: emptyList()
    }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val chains = accounts.map { accounts -> accounts.map(Account::chain) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun addressFor(chain: Chain): String = accounts.value.firstOrNull { it.chain == chain }?.address.orEmpty()
}

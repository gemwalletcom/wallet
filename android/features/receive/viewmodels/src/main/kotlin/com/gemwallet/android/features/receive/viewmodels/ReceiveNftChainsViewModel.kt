package com.gemwallet.android.features.receive.viewmodels

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.snapshotFlow
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.toChain
import com.gemwallet.android.ext.isNftSupported
import com.wallet.core.primitives.Chain
import dagger.hilt.android.lifecycle.HiltViewModel
import uniffi.gemstone.GemChainService
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import javax.inject.Inject

@HiltViewModel
class ReceiveNftChainsViewModel @Inject constructor(
    getSession: GetSession,
    private val chainService: GemChainService,
) : ViewModel() {

    private val wallet = getSession().value?.wallet

    private val allChains: List<Chain> = Chain.entries
        .filter { it.isNftSupported() }
        .filter { chain -> wallet?.getAccount(chain) != null }

    val chainFilter = TextFieldState()

    val chains = snapshotFlow { chainFilter.text.toString() }
        .map { query ->
            chainService.getMatchingChains(allChains.map { it.string }, query).mapNotNull { it.toChain() }
        }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, allChains)

    fun addressFor(chain: Chain): String = wallet?.getAccount(chain)?.address.orEmpty()
}

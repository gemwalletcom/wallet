package com.gemwallet.android.features.assets.viewmodels

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.coordinators.GetWalletSummary
import com.gemwallet.android.application.assets.coordinators.ToggleHideBalances
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.ext.isDefiSupported
import com.gemwallet.android.ext.isNftSupported
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import javax.inject.Inject

@HiltViewModel
class WalletHeaderViewModel @Inject constructor(
    private val toggleHideBalances: ToggleHideBalances,
    getWalletSummary: GetWalletSummary,
    getSession: GetSession,
) : ViewModel() {

    val walletSummary = getWalletSummary.getWalletSummary()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val availableContentTypes = getSession()
        .map { session -> session?.wallet?.availableContentTypes() ?: listOf(WalletContentType.Assets) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, listOf(WalletContentType.Assets))

    fun hideBalances() = viewModelScope.launch {
        toggleHideBalances()
    }
}

private fun Wallet.availableContentTypes(): List<WalletContentType> =
    WalletContentType.entries.filter { isAvailable(it) }

private fun Wallet.isAvailable(type: WalletContentType): Boolean = when (type) {
    WalletContentType.Assets -> true
    WalletContentType.Collections -> isChainSupported { it.isNftSupported() }
    WalletContentType.Defi -> isChainSupported { it.isDefiSupported() }
}

private fun Wallet.isChainSupported(flag: (Chain) -> Boolean): Boolean = when (type) {
    WalletType.Multicoin -> true
    WalletType.Single,
    WalletType.PrivateKey,
    WalletType.View -> accounts.firstOrNull()?.chain?.let(flag) ?: false
}

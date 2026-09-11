package com.gemwallet.android.data.coordinators.wallet

import com.gemwallet.android.ext.requireChain
import com.gemwallet.android.ext.toGem
import androidx.compose.runtime.Stable
import com.gemwallet.android.application.wallet.cases.GetWalletDetails
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import com.gemwallet.android.domains.wallet.aggregates.WalletDetailsAggregate
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.Wallet
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.GemWalletSecretKind
import uniffi.gemstone.walletPrivateKeyChains
import uniffi.gemstone.walletRow
import uniffi.gemstone.walletSecretKind
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.mapLatest

@OptIn(ExperimentalCoroutinesApi::class)
class GetWalletDetailsImpl(
    private val walletStore: GemstoneWalletStore
) : GetWalletDetails {

    override fun getWallet(walletId: WalletId): Flow<WalletDetailsAggregate?> {
        return  walletStore.observeWallet(walletId)
            .mapLatest { dto -> dto?.let { WalletDetailsAggregateImpl(it) } }
    }
}

@Stable
class WalletDetailsAggregateImpl(wallet: Wallet) : WalletDetailsAggregate {
    private val gemWallet = wallet.toGem()
    override val id: WalletId = wallet.id
    override val name: String = wallet.name
    override val secretKind: GemWalletSecretKind? = walletSecretKind(gemWallet)
    override val privateKeyChains: List<Chain> = walletPrivateKeyChains(gemWallet).map { it.requireChain() }
    override val row: GemWalletRow = walletRow(gemWallet)
    override val accounts: List<ChainAddress> = wallet.accounts.map {
        ChainAddress(chain = it.chain, address = it.address)
    }
    override val imageUrl: String? = wallet.imageUrl
}

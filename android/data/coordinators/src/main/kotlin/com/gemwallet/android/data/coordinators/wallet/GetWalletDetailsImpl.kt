package com.gemwallet.android.data.coordinators.wallet

import androidx.compose.runtime.Stable
import com.gemwallet.android.application.wallet.cases.GetWalletDetails
import com.gemwallet.android.data.services.gemstone.stores.GemstoneWalletStore
import com.gemwallet.android.domains.wallet.aggregates.WalletDetailsAggregate
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.BlockExplorerLink
import com.wallet.core.primitives.ChainAddress
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.mapLatest
import uniffi.gemstone.GemWalletDetails
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.GemWalletSecretKind
import uniffi.gemstone.GemWalletService

@OptIn(ExperimentalCoroutinesApi::class)
class GetWalletDetailsImpl(private val walletStore: GemstoneWalletStore, private val walletService: GemWalletService) : GetWalletDetails {

    override fun getWallet(walletId: WalletId): Flow<WalletDetailsAggregate?> = walletStore.observeWallet(walletId)
        .mapLatest { dto -> dto?.let { WalletDetailsAggregateImpl(walletService.walletDetails(it.toGem())) } }
}

@Stable
class WalletDetailsAggregateImpl(details: GemWalletDetails) : WalletDetailsAggregate {
    override val row: GemWalletRow = details.row
    override val id: WalletId = WalletId(details.row.id)
    override val secretKind: GemWalletSecretKind? = details.secretKind
    override val address: ChainAddress? = details.address?.toPrimitives()
    override val addressExplorer: BlockExplorerLink? = details.addressExplorer?.toPrimitives()
}

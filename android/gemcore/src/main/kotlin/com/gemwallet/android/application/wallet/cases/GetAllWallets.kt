package com.gemwallet.android.application.wallet.cases

import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import kotlinx.coroutines.flow.StateFlow

interface GetAllWallets {
    fun getAllWallets(): StateFlow<List<WalletDataAggregate>>
}

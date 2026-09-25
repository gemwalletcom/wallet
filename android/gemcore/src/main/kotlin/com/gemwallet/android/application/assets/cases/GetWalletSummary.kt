package com.gemwallet.android.application.assets.cases

import com.gemwallet.android.domains.wallet.aggregates.WalletSummary
import kotlinx.coroutines.flow.Flow

interface GetWalletSummary {
    fun getWalletSummary(): Flow<WalletSummary?>
}

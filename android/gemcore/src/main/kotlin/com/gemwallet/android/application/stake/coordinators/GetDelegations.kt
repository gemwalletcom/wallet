package com.gemwallet.android.application.stake.coordinators

import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface GetDelegations {
    operator fun invoke(walletId: WalletId, assetId: AssetId): Flow<List<Delegation>>
}

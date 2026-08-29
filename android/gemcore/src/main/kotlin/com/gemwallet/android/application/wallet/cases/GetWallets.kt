package com.gemwallet.android.application.wallet.cases

import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.Flow

interface GetWallets {
    operator fun invoke(): Flow<List<Wallet>>
}

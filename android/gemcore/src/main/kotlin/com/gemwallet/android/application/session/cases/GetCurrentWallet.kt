package com.gemwallet.android.application.session.cases

import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.Flow

interface GetCurrentWallet {
    suspend fun getCurrentWallet(): Wallet?

    fun observe(): Flow<Wallet?>
}

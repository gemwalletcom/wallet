package com.gemwallet.android.application.recipient.coordinators

import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.flow.Flow

interface GetWallets {
    fun getAll(): Flow<List<Wallet>>
}

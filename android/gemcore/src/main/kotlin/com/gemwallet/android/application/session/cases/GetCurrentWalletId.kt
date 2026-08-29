package com.gemwallet.android.application.session.cases

import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface GetCurrentWalletId {
    operator fun invoke(): Flow<WalletId>
}

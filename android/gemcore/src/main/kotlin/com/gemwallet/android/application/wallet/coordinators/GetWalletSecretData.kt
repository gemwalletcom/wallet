package com.gemwallet.android.application.wallet.coordinators

import com.gemwallet.android.domains.wallet.values.WalletSecretDataValue
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface GetWalletSecretData {
    fun getSecretData(walletId: WalletId): Flow<WalletSecretDataValue>
}

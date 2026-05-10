package com.gemwallet.android.application.wallet_import.coordinators

import com.gemwallet.android.application.wallet_import.values.ImportWalletState
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

interface GetImportWalletState {
    fun getImportState(walletId: WalletId): Flow<ImportWalletState>
}
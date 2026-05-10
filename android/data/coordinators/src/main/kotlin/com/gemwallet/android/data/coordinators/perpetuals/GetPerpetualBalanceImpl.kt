package com.gemwallet.android.data.coordinators.perpetuals

import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualBalance
import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.ext.HypercoreUSDC
import com.wallet.core.primitives.PerpetualBalance
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow

class GetPerpetualBalanceImpl(
    private val perpetualRepository: PerpetualRepository,
) : GetPerpetualBalance {
    override fun getBalance(walletId: WalletId): Flow<PerpetualBalance?> =
        perpetualRepository.getBalance(walletId, HypercoreUSDC.id)
}

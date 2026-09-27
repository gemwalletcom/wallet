package com.gemwallet.android.data.services.store.queries

import com.gemwallet.android.data.services.store.database.BalancesDao
import com.gemwallet.android.data.services.store.database.PricesDao
import com.gemwallet.android.ext.toIdentifier
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.PerpetualBalance
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.combine
import javax.inject.Inject

data class PerpetualWalletBalance(val balance: PerpetualBalance, val price: Double)

class PerpetualWalletBalanceQuery @Inject constructor(private val balancesDao: BalancesDao, private val pricesDao: PricesDao) {

    operator fun invoke(walletId: WalletId, assetId: AssetId): Flow<PerpetualWalletBalance?> = combine(
        balancesDao.perpetualBalance(walletId.id, assetId.toIdentifier()),
        pricesDao.getPrice(assetId.toIdentifier()),
    ) { balance, price ->
        balance?.let { PerpetualWalletBalance(balance = PerpetualBalance(available = it.available, reserved = it.reserved, withdrawable = it.withdrawable), price = price ?: 0.0) }
    }
}

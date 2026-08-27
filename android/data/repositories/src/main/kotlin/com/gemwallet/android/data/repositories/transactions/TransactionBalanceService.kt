package com.gemwallet.android.data.repositories.transactions

import com.gemwallet.android.data.repositories.perpetual.PerpetualRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.HypercoreUSDC
import com.gemwallet.android.domains.confirm.toTransferData
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import kotlinx.coroutines.flow.firstOrNull
import uniffi.gemstone.GemTransferBalance
import uniffi.gemstone.GemTransferService
import java.math.BigInteger
import javax.inject.Inject

class TransactionBalanceService @Inject constructor(
    private val perpetualRepository: PerpetualRepository,
    private val sessionRepository: SessionRepository,
) {

    suspend fun getBalance(assetInfo: AssetInfo, params: ConfirmParams): BigInteger {
        if (params is ConfirmParams.PerpetualParams) {
            return getPerpetualBalance(assetInfo)
        }
        val balance = assetInfo.balance.balance
        return GemTransferService().availableValue(
            params.toTransferData(),
            GemTransferBalance(
                available = balance.available,
                frozen = balance.frozen,
                locked = balance.locked,
                withdrawable = balance.withdrawable,
                votes = assetInfo.balance.metadata?.votes ?: 0u,
            ),
        ).toBigIntegerOrNull() ?: BigInteger.ZERO
    }

    private suspend fun getPerpetualBalance(assetInfo: AssetInfo): BigInteger {
        val walletId = sessionRepository.session().value?.wallet?.id ?: return BigInteger.ZERO
        val amount = perpetualRepository.getBalance(walletId, HypercoreUSDC.id).firstOrNull()?.available ?: 0.0
        return Crypto(amount.toBigDecimal(), assetInfo.asset.decimals).atomicValue
    }
}

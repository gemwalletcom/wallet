package com.gemwallet.android.data.services.gemstone.transactions

import com.gemwallet.android.data.services.gemstone.stores.GemstonePerpetualStore
import com.gemwallet.android.application.session.cases.GetSession
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
    private val perpetualStore: GemstonePerpetualStore,
    private val getSession: GetSession,
    private val transferService: GemTransferService,
) {

    suspend fun getBalance(assetInfo: AssetInfo, params: ConfirmParams): BigInteger {
        val balance = assetInfo.balance.balance
        val available = if (params is ConfirmParams.PerpetualParams) perpetualAvailable(assetInfo) else balance.available
        return transferService.availableValue(
            params.toTransferData(),
            GemTransferBalance(
                available = available,
                frozen = balance.frozen,
                locked = balance.locked,
                withdrawable = balance.withdrawable,
                votes = assetInfo.balance.metadata?.votes ?: 0u,
            ),
        ).toBigInteger()
    }

    private suspend fun perpetualAvailable(assetInfo: AssetInfo): String {
        val walletId = getSession().value?.wallet?.id ?: return "0"
        val amount = perpetualStore.observeBalance(walletId, HypercoreUSDC.id).firstOrNull()?.available ?: 0.0
        return Crypto(amount.toBigDecimal(), assetInfo.asset.decimals).atomicValue.toString()
    }
}

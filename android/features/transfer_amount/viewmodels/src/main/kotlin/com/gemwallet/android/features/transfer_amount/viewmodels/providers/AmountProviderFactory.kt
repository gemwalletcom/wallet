package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.coordinators.GetAssetInfo
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetual
import com.gemwallet.android.application.perpetual.coordinators.GetPerpetualBalance
import com.gemwallet.android.application.stake.coordinators.GetDelegation
import com.gemwallet.android.application.stake.coordinators.GetDelegations
import com.gemwallet.android.application.stake.coordinators.GetRecommendedValidator
import com.gemwallet.android.application.stake.coordinators.GetStakeValidator
import com.gemwallet.android.data.repositories.config.UserConfig
import com.gemwallet.android.data.repositories.transactions.TransactionBalanceService
import com.gemwallet.android.model.AmountParams
import kotlinx.coroutines.CoroutineScope
import javax.inject.Inject

class AmountProviderFactory @Inject constructor(
    private val transactionBalanceService: TransactionBalanceService,
    private val getAssetInfo: GetAssetInfo,
    private val getDelegation: GetDelegation,
    private val getDelegations: GetDelegations,
    private val getRecommendedValidator: GetRecommendedValidator,
    private val getStakeValidator: GetStakeValidator,
    private val getPerpetual: GetPerpetual,
    private val getPerpetualBalance: GetPerpetualBalance,
    private val userConfig: UserConfig,
) {
    fun create(params: AmountParams, scope: CoroutineScope): AmountDataProvider = when (params) {
        is AmountParams.Transfer,
        is AmountParams.Deposit,
        is AmountParams.Withdraw -> AmountTransferProvider(
            params = params,
            getAssetInfo = getAssetInfo,
            transactionBalanceService = transactionBalanceService,
            scope = scope,
        )
        is AmountParams.Stake -> AmountStakeProvider(
            params = params,
            getAssetInfo = getAssetInfo,
            getDelegation = getDelegation,
            getDelegations = getDelegations,
            getRecommendedValidator = getRecommendedValidator,
            getStakeValidator = getStakeValidator,
            transactionBalanceService = transactionBalanceService,
            scope = scope,
        )
        is AmountParams.Perpetual -> AmountPerpetualProvider(
            params = params,
            userConfig = userConfig,
            getAssetInfo = getAssetInfo,
            getPerpetual = getPerpetual,
            getPerpetualBalance = getPerpetualBalance,
            scope = scope,
        )
    }
}

package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.perpetual.cases.GetPerpetual
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.stake.cases.GetDelegation
import com.gemwallet.android.application.stake.cases.GetDelegations
import com.gemwallet.android.application.stake.cases.GetRecommendedValidator
import com.gemwallet.android.application.stake.cases.GetStakeValidator
import com.gemwallet.android.model.AmountParams
import kotlinx.coroutines.CoroutineScope
import uniffi.gemstone.GemAmountServiceInterface
import javax.inject.Inject

class AmountProviderFactory @Inject constructor(
    private val getAssetInfo: GetAssetInfo,
    private val getDelegation: GetDelegation,
    private val getDelegations: GetDelegations,
    private val getRecommendedValidator: GetRecommendedValidator,
    private val getStakeValidator: GetStakeValidator,
    private val getPerpetual: GetPerpetual,
    private val getPerpetualBalance: GetPerpetualBalance,
    private val service: GemAmountServiceInterface,
) {
    fun create(params: AmountParams, scope: CoroutineScope): AmountDataProvider = when (params) {
        is AmountParams.Transfer,
        is AmountParams.Deposit,
        is AmountParams.Withdraw -> AmountTransferProvider(
            params = params,
            getAssetInfo = getAssetInfo,
            scope = scope,
        )
        is AmountParams.Stake -> AmountStakeProvider(
            params = params,
            getAssetInfo = getAssetInfo,
            getDelegation = getDelegation,
            getDelegations = getDelegations,
            getRecommendedValidator = getRecommendedValidator,
            getStakeValidator = getStakeValidator,
            service = service,
            scope = scope,
        )
        is AmountParams.Perpetual -> AmountPerpetualProvider(
            params = params,
            service = service,
            getAssetInfo = getAssetInfo,
            getPerpetual = getPerpetual,
            getPerpetualBalance = getPerpetualBalance,
            scope = scope,
        )
    }
}

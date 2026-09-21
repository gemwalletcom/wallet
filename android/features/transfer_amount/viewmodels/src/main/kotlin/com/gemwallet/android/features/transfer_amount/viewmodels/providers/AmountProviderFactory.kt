package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import android.content.Context
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.perpetual.cases.GetPerpetual
import com.gemwallet.android.application.perpetual.cases.GetPerpetualBalance
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.stake.cases.GetDelegation
import com.gemwallet.android.application.stake.cases.GetStakeValidator
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.model.AmountParams
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineScope
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemStakeServiceInterface
import javax.inject.Inject

class AmountProviderFactory @Inject constructor(
    private val getAssetInfo: GetAssetInfo,
    private val getDelegation: GetDelegation,
    private val getStakeValidator: GetStakeValidator,
    private val getValidators: GetValidators,
    private val getPerpetual: GetPerpetual,
    private val getPerpetualBalance: GetPerpetualBalance,
    private val getSession: GetSession,
    private val service: GemAmountServiceInterface,
    private val stakeService: GemStakeServiceInterface,
    @param:ApplicationContext private val context: Context,
) {
    fun create(params: AmountParams, scope: CoroutineScope): AmountDataProvider = when (params) {
        is AmountParams.Transfer,
        is AmountParams.Deposit,
        is AmountParams.Withdraw,
        -> AmountTransferProvider(
            params = params,
            service = service,
            getAssetInfo = getAssetInfo,
            scope = scope,
        )

        is AmountParams.Stake -> AmountStakeProvider(
            params = params,
            getAssetInfo = getAssetInfo,
            getDelegation = getDelegation,
            getStakeValidator = getStakeValidator,
            getValidators = getValidators,
            stakeService = stakeService,
            scope = scope,
        )

        is AmountParams.Earn -> AmountEarnProvider(
            params = params,
            getAssetInfo = getAssetInfo,
            getDelegation = getDelegation,
            getStakeValidator = getStakeValidator,
            getSession = getSession,
            service = service,
            scope = scope,
        )

        is AmountParams.Perpetual -> AmountPerpetualProvider(
            params = params,
            context = context,
            service = service,
            getAssetInfo = getAssetInfo,
            getPerpetual = getPerpetual,
            getPerpetualBalance = getPerpetualBalance,
            scope = scope,
        )
    }
}

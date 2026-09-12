package com.gemwallet.android.features.transfer_amount.viewmodels.providers

import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.stake.cases.GetDelegation
import com.gemwallet.android.application.stake.cases.GetStakeValidator
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.features.transfer_amount.models.AmountError
import com.gemwallet.android.features.transfer_amount.viewmodels.AmountTitle
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.Crypto
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemAmountServiceInterface
import uniffi.gemstone.GemAmountType
import uniffi.gemstone.EarnType
import uniffi.gemstone.GemTransferData

@OptIn(ExperimentalCoroutinesApi::class)
class AmountEarnProvider(
    val params: AmountParams.Earn,
    getAssetInfo: GetAssetInfo,
    getDelegation: GetDelegation,
    private val getStakeValidator: GetStakeValidator,
    getSession: GetSession,
    private val service: GemAmountServiceInterface,
    scope: CoroutineScope,
) : AmountDataProvider(scope) {

    override val title: AmountTitle = AmountTitle.Earn(params)

    override val assetInfo: StateFlow<AssetInfo?> =
        getAssetInfo(params.assetId)
            .flowOn(Dispatchers.IO)
            .stateIn(scope, SharingStarted.Eagerly, null)

    private val earnType: StateFlow<EarnType?> = when (params) {
        is AmountParams.Earn.Deposit -> assetInfo
            .map { current -> current?.let { getStakeValidator(it.asset.id, params.providerId)?.toGem() } }
            .map { provider -> provider?.let { EarnType.Deposit(it) } }
        is AmountParams.Earn.Withdraw -> getSession()
            .flatMapLatest { session ->
                session?.wallet?.id?.let { getDelegation(it, params.validatorId, params.delegationId) } ?: flowOf(null)
            }
            .map { delegation -> delegation?.let { EarnType.Withdraw(it.toGem()) } }
    }
        .flowOn(Dispatchers.IO)
        .stateIn(scope, SharingStarted.Eagerly, null)

    override val amountType: StateFlow<GemAmountType?> =
        combine(earnType, assetInfo) { type, current ->
            if (type == null || current == null) null else service.earnAmountType(type)
        }
            .flowOn(Dispatchers.IO)
            .stateIn(scope, SharingStarted.Eagerly, null)

    override suspend fun buildTransfer(amount: Crypto, isMax: Boolean): GemTransferData {
        val current = assetInfo.value ?: error("assetInfo not loaded")
        val type = earnType.value ?: throw AmountError.NoValidatorSelected
        return service.earnTransferData(current.asset.toGem(), type, amount.atomicValue, isMax)
    }
}

package com.gemwallet.android.features.earn.delegation.viewmodels

import uniffi.gemstone.GemStakeServiceInterface
import com.gemwallet.android.ext.toCurrency
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.serializer.toJson
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.stake.cases.GetDelegation
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.stake.rewardsBalance
import com.gemwallet.android.ext.changeAmountOnUnstake
import com.gemwallet.android.model.AmountParams
import com.wallet.core.primitives.StakeType
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.ui.components.list_item.availableIn
import com.gemwallet.android.ui.models.RewardsInfoUIModel
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.features.earn.delegation.models.toDelegationAction
import com.gemwallet.android.features.earn.delegation.models.DelegationProperty
import com.gemwallet.android.features.earn.delegation.models.HeadDelegationInfo
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import java.math.BigInteger
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class DelegationViewModel @Inject constructor(
    private val getAssetInfo: GetAssetInfo,
    private val getDelegation: GetDelegation,
    private val stakeService: GemStakeServiceInterface,
    getSession: GetSession,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    val validatorId = MutableStateFlow(savedStateHandle.requireString(RouteArgument.ValidatorId))
    val delegationId = MutableStateFlow(savedStateHandle.getString(RouteArgument.DelegationId))

    val delegation = combine(
        validatorId,
        delegationId,
        getSession().filterNotNull(),
    ) { validatorId, delegationId, session -> Triple(validatorId, delegationId, session.wallet.id) }
        .flatMapLatest { (validatorId, delegationId, walletId) ->
            getDelegation(walletId = walletId, validatorId = validatorId, delegationId = delegationId)
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val assetInfo = delegation.filterNotNull()
        .flatMapLatest { getAssetInfo(it.base.assetId) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val properties = combine(
        delegation,
        assetInfo,
    ) { delegation, assetInfo ->
        if (delegation == null || assetInfo == null) {
            return@combine emptyList()
        }
        val availableIn = availableIn(delegation)
        val validatorUrl = stakeService.validatorUrl(delegation.validator.toJson())?.link
        listOfNotNull(
            DelegationProperty.Name(delegation.validator.name, validatorUrl),
            delegation.validator.takeIf { it.apr != 0.0 }?.let { DelegationProperty.Apr(it) },
            DelegationProperty.TransactionStatus(delegation.base.state, delegation.validator.isActive),
            delegation.base.state
                .takeIf { stakeService.showsCompletionDate(delegation.base.toJson()) && availableIn.isNotEmpty() }
                ?.let { DelegationProperty.State(it, availableIn) }
        )
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val balances = combine(
        delegation,
        assetInfo,
    ) { delegation, assetInfo ->
        if (delegation == null || assetInfo == null) {
            return@combine emptyList()
        }

        listOfNotNull(
            delegation.base.rewards
                .takeIf { stakeService.showsRewards(delegation.base.toJson()) }
                ?.let { RewardsInfoUIModel(assetInfo, it) },
        )
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val actions = combine(delegation.filterNotNull(), getSession().filterNotNull()) { delegation, session ->
        stakeService.delegationActions(session.wallet.type.toGem(), delegation.toJson()).mapNotNull { it.toDelegationAction() }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val canClaimRewards = combine(delegation.filterNotNull(), getSession().filterNotNull()) { delegation, session ->
        stakeService.canClaimDelegationRewards(session.wallet.type.toGem(), delegation.toJson())
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val delegationInfo = combine(delegation, assetInfo) { delegation, assetInfo ->
        if (assetInfo == null || delegation == null) {
            return@combine null
        }
        HeadDelegationInfo(delegation, assetInfo, stakeService.currency().toCurrency())
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun onStake(call: AmountTransactionAction) {
        buildDelegate()?.let { call(it) }
    }

    fun onUnstake(amountCall: AmountTransactionAction, confirmCall: ConfirmTransactionAction) {
        val assetInfo = assetInfo.value ?: return
        val delegation = delegation.value ?: return
        if (assetInfo.chain.changeAmountOnUnstake) {
            buildUndelegate()?.let { amountCall(it) }
            return
        }
        val balance = Crypto(delegation.base.balance.toBigIntegerOrNull() ?: BigInteger.ZERO)
        confirmCall(stakeService.stakeTransferData(assetInfo.asset.toGem(), StakeType.Unstake(delegation).toJson(), balance.atomicValue.toString(), false))
    }

    fun onRedelegate(call: AmountTransactionAction) {
        buildRedelegate()?.let { call(it) }
    }

    fun onWithdraw(call: ConfirmTransactionAction) {
        val assetInfo = assetInfo.value ?: return
        val delegation = delegation.value ?: return
        val balance = Crypto(delegation.base.balance.toBigIntegerOrNull() ?: BigInteger.ZERO)
        call(stakeService.stakeTransferData(assetInfo.asset.toGem(), StakeType.Withdraw(delegation).toJson(), balance.atomicValue.toString(), false))
    }

    fun onClaimRewards(call: ConfirmTransactionAction) {
        val assetInfo = assetInfo.value ?: return
        val delegation = delegation.value ?: return
        call(
            stakeService.stakeTransferData(
                assetInfo.asset.toGem(),
                StakeType.Rewards(listOf(delegation.validator)).toJson(),
                delegation.rewardsBalance().toString(),
                false,
            )
        )
    }

    private fun buildDelegate(): AmountParams.Stake.Delegate? {
        val assetId = assetInfo.value?.asset?.id ?: return null
        val delegation = delegation.value ?: return null
        return AmountParams.Stake.Delegate(assetId, validatorId = delegation.validator.id)
    }

    private fun buildUndelegate(): AmountParams.Stake.Undelegate? {
        val assetId = assetInfo.value?.asset?.id ?: return null
        val delegation = delegation.value ?: return null
        return AmountParams.Stake.Undelegate(
            assetId = assetId,
            validatorId = delegation.validator.id,
            delegationId = delegation.base.delegationId,
        )
    }

    private fun buildRedelegate(): AmountParams.Stake.Redelegate? {
        val assetId = assetInfo.value?.asset?.id ?: return null
        val delegation = delegation.value ?: return null
        return AmountParams.Stake.Redelegate(
            assetId = assetId,
            validatorId = delegation.validator.id,
            delegationId = delegation.base.delegationId,
        )
    }
}

private fun SavedStateHandle.requireString(argument: RouteArgument): String {
    val value = checkNotNull(get<String>(argument.key)) { "Missing route argument: ${argument.key}" }
    check(value.isNotBlank()) { "Blank route argument: ${argument.key}" }
    return value
}

private fun SavedStateHandle.getString(argument: RouteArgument): String =
    get<String>(argument.key).orEmpty()

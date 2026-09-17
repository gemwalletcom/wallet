package com.gemwallet.android.features.earn.delegation.viewmodels

import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.stake.cases.GetDelegation
import com.gemwallet.android.data.services.gemstone.di.IoDispatcher
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.earn.delegation.models.DelegationProperties
import com.gemwallet.android.features.earn.delegation.models.HeadDelegationInfo
import com.gemwallet.android.features.earn.delegation.models.uiModel
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.toAmountParams
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ui.components.list_item.availableIn
import com.gemwallet.android.ui.models.RewardsInfoUIModel
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.StakeType
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import java.math.BigInteger
import javax.inject.Inject
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemDelegationAction
import uniffi.gemstone.GemDelegationDestination
import uniffi.gemstone.GemStakeServiceInterface
import uniffi.gemstone.delegationStatus

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class DelegationViewModel @Inject constructor(
    private val getAssetInfo: GetAssetInfo,
    private val getDelegation: GetDelegation,
    private val stakeService: GemStakeServiceInterface,
    getSession: GetSession,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    savedStateHandle: SavedStateHandle,
    @param:ApplicationContext private val context: Context,
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
            return@combine null
        }
        val validatorName = stakeService.validatorRow(delegation.validator.toGem()).name
        val validatorUrl = stakeService.validatorUrl(delegation.validator.toGem())?.link
        val status = delegationStatus(delegation.toGem())
        val availableIn = availableIn(delegation)
        DelegationProperties(
            rows = stakeService.delegationRows(delegation.toGem()).mapNotNull { it.uiModel(context, delegation.validator, validatorName, validatorUrl, status, availableIn) },
            rewards = RewardsInfoUIModel(assetInfo, delegation.base.rewards),
        )
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val actions = combine(delegation.filterNotNull(), getSession().filterNotNull()) { delegation, session ->
        stakeService.delegationActions(session.wallet.type.toGem(), delegation.toGem()).map { it.uiModel(context) }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val canClaimRewards = combine(delegation.filterNotNull(), getSession().filterNotNull()) { delegation, session ->
        stakeService.canClaimDelegationRewards(session.wallet.type.toGem(), delegation.toGem())
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val delegationInfo = combine(delegation, assetInfo) { delegation, assetInfo ->
        if (assetInfo == null || delegation == null) {
            return@combine null
        }
        HeadDelegationInfo(delegation, assetInfo, stakeService.getCurrency().toPrimitives(), stakeService.validatorRow(delegation.validator.toGem()))
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun onAction(action: GemDelegationAction, onAmount: AmountTransactionAction, onConfirm: ConfirmTransactionAction) {
        val assetInfo = assetInfo.value ?: return
        val delegation = delegation.value ?: return
        when (val destination = stakeService.delegationActionDestination(assetInfo.asset.toGem(), delegation.toGem(), action, emptyList())) {
            GemDelegationDestination.Details -> Unit
            is GemDelegationDestination.Confirm -> onConfirm(ConfirmTransferInput(destination.transfer))
            is GemDelegationDestination.Amount -> onAmount(destination.input.toAmountParams(destination.asset.toPrimitives().id))
        }
    }

    fun onClaimRewards(call: ConfirmTransactionAction) {
        val assetInfo = assetInfo.value ?: return
        val delegation = delegation.value ?: return
        viewModelScope.launch {
            val transfer = withContext(ioDispatcher) {
                stakeService.stakeTransferData(
                    assetInfo.asset.toGem(),
                    StakeType.Rewards(listOf(delegation.validator)).toGem(),
                    delegation.base.rewards,
                    false,
                )
            }
            call(ConfirmTransferInput(transfer))
        }
    }
}

private fun SavedStateHandle.requireString(argument: RouteArgument): String {
    val value = checkNotNull(get<String>(argument.key)) { "Missing route argument: ${argument.key}" }
    check(value.isNotBlank()) { "Blank route argument: ${argument.key}" }
    return value
}

private fun SavedStateHandle.getString(argument: RouteArgument): String =
    get<String>(argument.key).orEmpty()

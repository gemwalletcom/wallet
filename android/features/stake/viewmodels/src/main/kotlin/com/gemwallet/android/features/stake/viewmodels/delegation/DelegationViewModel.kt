package com.gemwallet.android.features.stake.viewmodels.delegation

import android.content.Context
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetQuery
import com.gemwallet.android.data.services.store.queries.DelegationQuery
import com.gemwallet.android.data.services.store.queries.ValidatorsQuery
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.stake.viewmodels.delegation.models.DelegationProperties
import com.gemwallet.android.features.stake.viewmodels.delegation.models.DelegationRowUIModel
import com.gemwallet.android.features.stake.viewmodels.delegation.models.uiModel
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.toAmountParams
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.StakeType
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.GemDelegationDestination
import uniffi.gemstone.GemStakeServiceInterface
import java.math.BigInteger
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class DelegationViewModel @Inject constructor(
    getCurrentWalletId: GetCurrentWalletId,
    assetQuery: AssetQuery,
    private val delegationQuery: DelegationQuery,
    validatorsQuery: ValidatorsQuery,
    private val stakeService: GemStakeServiceInterface,
    getSession: GetSession,
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
            delegationQuery(walletId = walletId, validatorId = validatorId, delegationId = delegationId)
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val assetInfo = delegation.filterNotNull()
        .flatMapLatest { delegation -> getCurrentWalletId().flatMapLatest { walletId -> assetQuery(walletId.id, delegation.base.assetId) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val validators = delegation.filterNotNull()
        .flatMapLatest { validatorsQuery(it.base.assetId, StakeProviderType.Stake) }

    val properties = combine(
        delegation,
        assetInfo,
        getSession().filterNotNull(),
        validators,
    ) { delegation, assetInfo, session, validators ->
        if (delegation == null || assetInfo == null) {
            return@combine null
        }
        val details = stakeService.delegationDetails(
            session.wallet.type.toGem(),
            delegation.toGem(),
            assetInfo.asset.toGem(),
            assetInfo.price?.price?.price,
            (assetInfo.price?.currency ?: Currency.USD).toGem(),
            validators.map { it.toGem() },
        )
        DelegationProperties(
            rows = details.rows.map { DelegationRowUIModel.Row(it) } + listOfNotNull(DelegationRowUIModel.Rewards.takeIf { details.rewards != null }),
            details = details,
            actions = details.actions.map { it.uiModel(context) },
            asset = assetInfo.asset,
        )
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun onAction(destination: GemDelegationDestination, onAmount: AmountTransactionAction, onConfirm: ConfirmTransactionAction) {
        when (destination) {
            GemDelegationDestination.Details -> Unit
            is GemDelegationDestination.Confirm -> onConfirm(ConfirmTransferInput(destination.transfer))
            is GemDelegationDestination.Amount -> onAmount(destination.input.toAmountParams(destination.asset.toPrimitives().id))
        }
    }

    fun onClaimRewards(call: ConfirmTransactionAction) {
        val transfer = properties.value?.details?.claim ?: return
        call(ConfirmTransferInput(transfer))
    }
}

private fun SavedStateHandle.requireString(argument: RouteArgument): String {
    val value = checkNotNull(get<String>(argument.key)) { "Missing route argument: ${argument.key}" }
    check(value.isNotBlank()) { "Blank route argument: ${argument.key}" }
    return value
}

private fun SavedStateHandle.getString(argument: RouteArgument): String = get<String>(argument.key).orEmpty()

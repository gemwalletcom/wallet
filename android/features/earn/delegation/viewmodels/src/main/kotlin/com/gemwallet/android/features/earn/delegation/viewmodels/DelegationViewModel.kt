package com.gemwallet.android.features.earn.delegation.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.stake.StakeRepository
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.stake.hasRewards
import com.gemwallet.android.domains.stake.rewardsBalance
import com.gemwallet.android.ext.byChain
import com.gemwallet.android.ext.canClaimRewards
import com.gemwallet.android.ext.changeAmountOnUnstake
import com.gemwallet.android.ext.redelegated
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.ConfirmParams
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.ui.components.list_item.availableIn
import com.gemwallet.android.ui.models.RewardsInfoUIModel
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.features.earn.delegation.models.DelegationActions
import com.gemwallet.android.features.earn.delegation.models.DelegationProperty
import com.gemwallet.android.features.earn.delegation.models.HeadDelegationInfo
import com.wallet.core.primitives.DelegationState
import com.wallet.core.primitives.StakeChain
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.WalletType
import dagger.hilt.android.lifecycle.HiltViewModel
import uniffi.gemstone.Explorer
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import java.math.BigInteger
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class DelegationViewModel @Inject constructor(
    private val assetsRepository: AssetsRepository,
    private val stakeRepository: StakeRepository,
    private val getCurrentBlockExplorer: GetCurrentBlockExplorer,
    sessionRepository: SessionRepository,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    val validatorId = savedStateHandle.getStateFlow<String?>("validatorId", null).filterNotNull()
    val delegationId = savedStateHandle.getStateFlow<String?>("delegationId", null).filterNotNull()
    val state = savedStateHandle.getStateFlow<String?>("state", null).filterNotNull()
        .map { DelegationState.valueOf(it) }

    val delegation = combine(validatorId, delegationId, state) { validatorId, delegationId, state -> Triple(validatorId, delegationId, state) }
        .flatMapLatest {
            val (validatorId, delegationId, state) = it
            stakeRepository.getDelegation(delegationId = delegationId, validatorId = validatorId, state = state)
        }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val assetInfo = delegation.filterNotNull()
        .flatMapLatest { assetsRepository.getAssetInfo(it.base.assetId) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val properties = combine(
        delegation,
        assetInfo,
    ) { delegation, assetInfo ->
        if (delegation == null || assetInfo == null) {
            return@combine emptyList()
        }
        val availableIn = availableIn(delegation)
        val chain = delegation.validator.chain
        val validatorUrl = Explorer(chain.string)
            .getValidatorUrl(getCurrentBlockExplorer.getCurrentBlockExplorer(chain), delegation.validator.id)
        listOfNotNull(
            DelegationProperty.Name(delegation.validator.name, validatorUrl),
            DelegationProperty.Apr(delegation.validator),
            DelegationProperty.TransactionStatus(delegation.base.state, delegation.validator.isActive),
            delegation.base.state
                .takeIf {
                    (it == DelegationState.Pending
                        || it == DelegationState.Activating
                        || it == DelegationState.Deactivating)
                        && availableIn.isNotEmpty()
                }
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
                .takeIf { it.toBigInteger() > BigInteger.ZERO }
                ?.let { RewardsInfoUIModel(assetInfo, it) },
        )
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val actions = combine(
        delegation,
        assetInfo,
        sessionRepository.session().filterNotNull(),
    ) { delegation, assetInfo, session ->
        if (delegation == null || assetInfo == null || session.wallet.type == WalletType.View) {
            return@combine emptyList()
        }
        val stakeChain = StakeChain.byChain(assetInfo.asset.id.chain)!!
        when (delegation.base.state) {
            DelegationState.Inactive,
            DelegationState.Active -> listOfNotNull(
                DelegationActions.StakeAction,
                DelegationActions.UnstakeAction,
                assetInfo.chain.takeIf { it.redelegated }?.let { DelegationActions.RedelegateAction },
            )
            DelegationState.AwaitingWithdrawal -> listOf( DelegationActions.WithdrawalAction )
            DelegationState.Pending,
            DelegationState.Activating,
            DelegationState.Deactivating -> emptyList()
        }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val canClaimRewards = combine(
        delegation,
        assetInfo,
        sessionRepository.session().filterNotNull(),
    ) { delegation, assetInfo, session ->
        if (delegation == null || assetInfo == null || session.wallet.type == WalletType.View) {
            return@combine false
        }
        assetInfo.asset.id.chain.canClaimRewards
            && delegation.hasRewards()
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    val delegationInfo = combine(
        delegation,
        assetInfo,
        sessionRepository.session().filterNotNull(),
    ) { delegation, assetInfo, session ->
        if (assetInfo == null || delegation == null) {
            return@combine null
        }
        HeadDelegationInfo(delegation, assetInfo)
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val uiState = combine(
        delegation,
        assetInfo,
        sessionRepository.session().filterNotNull(),
    ) { delegation, assetInfo, session ->
        if (assetInfo == null || delegation == null) {
            return@combine null
        }
        DelegationSceneState(
            walletType = session.wallet.type,
            state = delegation.base.state,
        )
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun onStake(call: AmountTransactionAction) {
        buildStake(TransactionType.StakeDelegate)?.let { call(it) }
    }

    fun onUnstake(amountCall: AmountTransactionAction, confirmCall: ConfirmTransactionAction) {
        val assetInfo = assetInfo.value ?: return
        val delegation = delegation.value ?: return
        if (assetInfo.chain.changeAmountOnUnstake) {
            buildStake(TransactionType.StakeUndelegate)?.let { amountCall(it) }
            return
        }
        val from = assetInfo.owner ?: return
        val balance = Crypto(delegation.base.balance.toBigIntegerOrNull() ?: BigInteger.ZERO)
        val params = ConfirmParams.Builder(assetInfo.asset, from, balance.atomicValue, false)
            .undelegate(delegation)
        confirmCall(params)
    }

    fun onRedelegate(call: AmountTransactionAction) {
        buildStake(TransactionType.StakeRedelegate)?.let { call(it) }
    }

    fun onWithdraw(call: ConfirmTransactionAction) {
        val assetInfo = assetInfo.value ?: return
        val from = assetInfo.owner ?: return
        val delegation = delegation.value ?: return
        val balance = Crypto(delegation.base.balance.toBigIntegerOrNull() ?: BigInteger.ZERO)
        val params = ConfirmParams.Builder(assetInfo.asset, from, balance.atomicValue, false)
            .withdraw(delegation)
        call(params)
    }

    fun onClaimRewards(call: ConfirmTransactionAction) {
        val assetInfo = assetInfo.value ?: return
        val from = assetInfo.owner ?: return
        val delegation = delegation.value ?: return
        call(
            ConfirmParams.Stake.RewardsParams(
                asset = assetInfo.asset,
                from = from,
                validators = listOf(delegation.validator),
                amount = delegation.rewardsBalance(),
            )
        )
    }

    private fun buildStake(type: TransactionType): AmountParams? {
        val assetId = assetInfo.value?.asset?.id ?: return null
        val delegation = delegation.value ?: return null
        return AmountParams.buildStake(
            assetId = assetId,
            txType = type,
            validatorId = delegation.validator.id,
            delegationId = delegation.base.delegationId,
            delegationState = delegation.base.state,
        )
    }
}
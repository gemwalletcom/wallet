package com.gemwallet.android.features.stake.viewmodels

import uniffi.gemstone.GemClaimRewardsDestination
import uniffi.gemstone.GemStakeServiceInterface
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.stake.cases.GetDelegations
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.application.stake.cases.SyncStakeDelegations
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.stakeChain
import com.gemwallet.android.AppUrl
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.serializer.toJson
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.model.AmountParams
import com.wallet.core.primitives.StakeType
import com.gemwallet.android.model.Crypto
import com.gemwallet.android.model.ValueFormatter
import com.gemwallet.android.model.toGem
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationState
import com.gemwallet.android.ext.isViewOnly
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import uniffi.gemstone.DocsUrl
import java.math.BigInteger
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class StakeViewModel @Inject constructor(
    private val getAssetInfo: GetAssetInfo,
    private val getDelegations: GetDelegations,
    private val getValidators: GetValidators,
    private val syncStakeDelegations: SyncStakeDelegations,
    private val stakeService: GemStakeServiceInterface,
    getSession: GetSession,
    stateHandle: SavedStateHandle,
): ViewModel() {
    private val initialAssetId = stateHandle.get<String>(RouteArgument.AssetId.key)?.toAssetId()
        ?: error("Missing assetId")

    private val assetId = stateHandle.getStateFlow(RouteArgument.AssetId.key, initialAssetId.toIdentifier())
        .map { it.toAssetId() ?: initialAssetId }
        .stateIn(viewModelScope, SharingStarted.Eagerly, initialAssetId)

    val assetInfo = assetId
        .flatMapLatest { getAssetInfo(it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val stakeInfoUrl = assetInfo
        .mapLatest { it?.stakeChain?.let { chain -> AppUrl.docs(DocsUrl.Staking(chain.string)) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val walletType = session.mapLatest { it?.wallet?.type }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val delegations = session.filterNotNull().combine(assetId) { session, assetId ->
        session.wallet.id to assetId
    }
        .flatMapLatest { (walletId, assetId) -> getDelegations(walletId, assetId) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val hasValidators = assetId
        .flatMapLatest { getValidators(it) }
        .mapLatest { validators -> validators.isNotEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, false)

    private val claimRewards = combine(delegations, assetInfo.filterNotNull()) { delegations, assetInfo ->
        stakeService.claimRewards(assetInfo.asset.chain.string, delegations.map { it.toJson() })
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val rewardsText = combine(claimRewards.filterNotNull(), assetInfo.filterNotNull()) { claimRewards, assetInfo ->
        ValueFormatter(style = ValueFormatter.Style.Auto).string(claimRewards.value, assetInfo.asset)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, "")

    val actions = combine(
        walletType.filterNotNull(),
        delegations,
        assetInfo.filterNotNull(),
        hasValidators,
    ) { walletType, delegations, assetInfo, hasValidators ->
        stakeService.stakeActions(
            walletType = walletType.toGem(),
            chain = assetInfo.asset.chain.string,
            hasValidators = hasValidators,
            balance = assetInfo.balance.toGem(),
            delegations = delegations.map { it.toJson() },
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val sync = MutableStateFlow<Boolean>(true)

    val isSync = sync
        .flatMapLatest { isSync ->
            flow {
                if (!isSync) {
                    emit(false)
                    return@flow
                }
                val assetInfo = assetInfo.filterNotNull().first()
                emit(true)
                syncStakeDelegations.sync(assetInfo.asset.id.chain)
                emit(false)
                sync.update { false }
            }
        }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    fun onRefresh() {
        sync.update { true }
    }

    fun onDelegation(
        delegation: Delegation,
        onOpenDetail: (String, String) -> Unit,
        onConfirm: ConfirmTransactionAction,
    ) {
        if (walletType.value?.isViewOnly == true || delegation.base.state != DelegationState.AwaitingWithdrawal) {
            onOpenDetail(delegation.validator.id, delegation.base.delegationId)
            return
        }
        val assetInfo = assetInfo.value ?: return
        onConfirm(stakeService.stakeTransferData(assetInfo.asset.toGem(), StakeType.Withdraw(delegation).toJson(), BigInteger(delegation.base.balance), false))
    }

    fun onRewards(onAmount: AmountTransactionAction, onConfirm: ConfirmTransactionAction) {
        val assetInfo = assetInfo.value ?: return
        when (val destination = claimRewards.value?.destination ?: return) {
            is GemClaimRewardsDestination.Transfer -> onConfirm(destination.transfer)
            is GemClaimRewardsDestination.Amount -> onAmount(AmountParams.Stake.Rewards(assetInfo.asset.id))
        }
    }
}

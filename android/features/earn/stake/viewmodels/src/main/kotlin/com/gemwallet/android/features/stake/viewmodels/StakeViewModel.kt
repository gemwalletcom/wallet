package com.gemwallet.android.features.stake.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.AppUrl
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.stake.cases.GetDelegations
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.stakeChain
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.features.stake.viewmodels.models.StakeActionUIModel
import com.gemwallet.android.features.stake.viewmodels.models.StakeSectionUIModel
import com.gemwallet.android.features.stake.viewmodels.models.uiModel
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.model.toAmountParams
import com.gemwallet.android.model.toGem
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Delegation
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
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
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemClaimRewardsDestination
import uniffi.gemstone.GemDelegationDestination
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemStakeServiceInterface
import uniffi.gemstone.validatorRow
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class StakeViewModel @Inject constructor(
    private val getAssetInfo: GetAssetInfo,
    private val getWalletAssets: GetWalletAssets,
    private val getDelegations: GetDelegations,
    private val getValidators: GetValidators,
    private val stakeService: GemStakeServiceInterface,
    getSession: GetSession,
    stateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {
    private val initialAssetId = stateHandle.get<String>(RouteArgument.AssetId.key)?.toAssetId()
        ?: error("Missing assetId")

    private val assetId = stateHandle.getStateFlow(RouteArgument.AssetId.key, initialAssetId.toIdentifier())
        .map { it.toAssetId() ?: initialAssetId }
        .stateIn(viewModelScope, SharingStarted.Eagerly, initialAssetId)

    val assetInfo = assetId
        .flatMapLatest { getAssetInfo(it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, getWalletAssets().value.firstOrNull { it.asset.id == initialAssetId })

    val stakeInfoUrl = assetInfo
        .mapLatest { it?.asset?.stakeChain?.let { chain -> AppUrl.staking(chain.string) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val infoRows: StateFlow<List<GemListRow>> = assetInfo
        .mapLatest { info -> info?.let { stakeService.stakeInfoRows(it.asset.toGem(), it.metadata.stakingApr) }.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val walletType = session.mapLatest { it?.wallet?.type }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val delegations = session.filterNotNull().combine(assetId) { session, assetId ->
        session.wallet.id to assetId
    }
        .flatMapLatest { (walletId, assetId) -> getDelegations(walletId, assetId) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val validatorRows = delegations
        .map { items -> items.associate { it.validator.id to validatorRow(it.validator.toGem()) } }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyMap())

    private val validators = assetId
        .flatMapLatest { getValidators(it) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val claimRewards = combine(delegations, assetInfo.filterNotNull()) { delegations, assetInfo ->
        stakeService.claimRewards(assetInfo.asset.chain.string, delegations.map { it.toGem() })
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val actions = combine(
        walletType.filterNotNull(),
        delegations,
        assetInfo.filterNotNull(),
        validators,
    ) { walletType, delegations, assetInfo, validators ->
        stakeService.stakeActions(
            walletType = walletType.toGem(),
            chain = assetInfo.asset.chain.string,
            validators = validators.map { it.toGem() },
            balance = assetInfo.balance.toGem(),
            delegations = delegations.map { it.toGem() },
        )
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val sections: StateFlow<List<StakeSectionUIModel>> = combine(assetInfo, actions, delegations, validatorRows) { assetInfo, actions, delegations, validatorRows ->
        assetInfo?.let { stakeService.stakeSections(it.asset.chain.string, actions.isNotEmpty(), delegations.isNotEmpty()) }.orEmpty()
            .map { it.uiModel(context, delegations, validatorRows) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val sync = MutableStateFlow<Boolean>(true)

    private val loadState = MutableStateFlow<GemLoadState>(GemLoadState.Loading)

    val loadError: StateFlow<GemServiceException?> = loadState
        .map { (it as? GemLoadState.Error)?.error }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val isSync = sync
        .flatMapLatest { isSync ->
            flow {
                if (!isSync) {
                    emit(false)
                    return@flow
                }
                val assetInfo = assetInfo.filterNotNull().first()
                emit(true)
                val state = withContext(ioDispatcher) { stakeService.refresh(assetInfo.asset.id.chain.string, delegations.value.map { it.toGem() }) }
                (state as? GemLoadState.Error)?.let { Log.e(TAG, "stake delegations sync failed", it.error) }
                loadState.value = state
                emit(false)
                sync.update { false }
            }
        }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    val actionRows: StateFlow<List<StakeActionUIModel>> = combine(actions, assetInfo.filterNotNull()) { actions, info ->
        actions.map { it.uiModel(context, info) }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun onRefresh() {
        sync.update { true }
    }

    fun onDelegation(delegation: Delegation, onOpenDetail: (String, String) -> Unit, onAmount: AmountTransactionAction, onConfirm: ConfirmTransactionAction) {
        val walletType = walletType.value ?: return
        val assetInfo = assetInfo.value ?: return
        when (val destination = stakeService.delegationDestination(walletType.toGem(), assetInfo.asset.toGem(), delegation.toGem())) {
            GemDelegationDestination.Details -> onOpenDetail(delegation.validator.id, delegation.base.delegationId)
            is GemDelegationDestination.Confirm -> onConfirm(ConfirmTransferInput(destination.transfer))
            is GemDelegationDestination.Amount -> onAmount(destination.input.toAmountParams(destination.asset.toPrimitives().id))
        }
    }

    fun onRewards(onAmount: AmountTransactionAction, onConfirm: ConfirmTransactionAction) {
        val assetInfo = assetInfo.value ?: return
        when (val destination = claimRewards.value?.destination ?: return) {
            is GemClaimRewardsDestination.Transfer -> onConfirm(ConfirmTransferInput(destination.transfer))
            is GemClaimRewardsDestination.Amount -> onAmount(AmountParams.Stake.Rewards(assetInfo.asset.id, destination.delegations))
        }
    }

    private companion object {
        const val TAG = "Stake"
    }
}

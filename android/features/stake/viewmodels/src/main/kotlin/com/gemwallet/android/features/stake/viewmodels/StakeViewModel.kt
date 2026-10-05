package com.gemwallet.android.features.stake.viewmodels

import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetQuery
import com.gemwallet.android.data.services.store.queries.DelegationsQuery
import com.gemwallet.android.data.services.store.queries.ValidatorsQuery
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.confirm.ConfirmTransferInput
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.StakeProviderType
import dagger.hilt.android.lifecycle.HiltViewModel
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
import uniffi.gemstone.GemInfoTopic
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemStakeActionKind
import uniffi.gemstone.GemStakeDestination
import uniffi.gemstone.GemStakeInput
import uniffi.gemstone.GemStakeServiceInterface
import uniffi.gemstone.GemStakeViewState
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class StakeViewModel @Inject constructor(
    getCurrentWalletId: GetCurrentWalletId,
    assetQuery: AssetQuery,
    private val getWalletAssets: GetWalletAssets,
    private val delegationsQuery: DelegationsQuery,
    private val validatorsQuery: ValidatorsQuery,
    private val service: GemStakeServiceInterface,
    getSession: GetSession,
    stateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {
    val infoSheet = MutableStateFlow<GemInfoTopic?>(null)

    private val initialAssetId = stateHandle.get<String>(RouteArgument.AssetId.key)?.toAssetId()
        ?: error("Missing assetId")

    private val assetId = stateHandle.getStateFlow(RouteArgument.AssetId.key, initialAssetId.toIdentifier())
        .map { it.toAssetId() ?: initialAssetId }
        .stateIn(viewModelScope, SharingStarted.Eagerly, initialAssetId)

    val assetInfo = assetId
        .flatMapLatest { assetId -> getCurrentWalletId().flatMapLatest { walletId -> assetQuery(walletId.id, assetId) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, getWalletAssets().value.firstOrNull { it.asset.id == initialAssetId })

    private val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val walletType = session.mapLatest { it?.wallet?.type }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val delegations = session.filterNotNull().combine(assetId) { session, assetId ->
        session.wallet.id to assetId
    }
        .flatMapLatest { (walletId, assetId) -> delegationsQuery(walletId, assetId, StakeProviderType.Stake) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val validators = assetId
        .flatMapLatest { validatorsQuery(it, StakeProviderType.Stake) }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val loadState = MutableStateFlow<GemLoadState>(GemLoadState.Loading)

    val viewState: StateFlow<GemStakeViewState?> = combine(
        walletType.filterNotNull(),
        delegations,
        assetInfo.filterNotNull(),
        validators,
        loadState,
    ) { walletType, delegations, assetInfo, validators, state ->
        service.stakeViewState(
            GemStakeInput(
                walletType = walletType.toGem(),
                assetData = assetInfo.toGem(),
                currency = service.getCurrency(),
                validators = validators.map { it.toGem() },
                delegations = delegations.map { it.toGem() },
                state = state,
            ),
        )
    }.flowOn(ioDispatcher).stateIn(viewModelScope, SharingStarted.Eagerly, null)

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
                val state = withContext(ioDispatcher) { service.refresh(assetInfo.asset.id.chain.string, delegations.value.map { it.toGem() }) }
                (state as? GemLoadState.Error)?.let { Log.e(TAG, "stake delegations sync failed", it.error) }
                loadState.value = state
                emit(false)
                sync.update { false }
            }
        }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    fun onRefresh() {
        sync.update { true }
    }

    fun onSelect(kind: GemStakeActionKind, destination: GemStakeDestination, onAmount: AmountTransactionAction, onConfirm: ConfirmTransactionAction) {
        when (kind) {
            GemStakeActionKind.STAKE -> if (!service.isAvailable()) {
                infoSheet.value = GemInfoTopic.RegionUnavailable
                return
            }

            GemStakeActionKind.FREEZE, GemStakeActionKind.UNFREEZE, GemStakeActionKind.CLAIM_REWARDS -> Unit
        }
        when (destination) {
            is GemStakeDestination.Amount -> onAmount(AmountParams.Stake(assetId.value, destination.input))
            is GemStakeDestination.Confirm -> onConfirm(ConfirmTransferInput(destination.transfer))
        }
    }

    fun onDelegation(delegation: Delegation, onOpenDetail: (String, String) -> Unit, onAmount: AmountTransactionAction, onConfirm: ConfirmTransactionAction) {
        val item = viewState.value?.delegations?.firstOrNull { it.delegation.toPrimitives() == delegation } ?: return
        item.destination.open(delegation, onOpenDetail, onAmount, onConfirm)
    }

    private companion object {
        const val TAG = "Stake"
    }
}

package com.gemwallet.android.features.stake.viewmodels

import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.session.cases.GetCurrentWalletId
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.data.services.store.queries.AssetQuery
import com.gemwallet.android.data.services.store.queries.DelegationsQuery
import com.gemwallet.android.data.services.store.queries.ValidatorsQuery
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.models.actions.AmountTransactionAction
import com.gemwallet.android.ui.models.actions.ConfirmTransactionAction
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletType
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
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
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import uniffi.gemstone.GemEarnInput
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemStakeServiceInterface
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class EarnViewModel @Inject constructor(
    getCurrentWalletId: GetCurrentWalletId,
    assetQuery: AssetQuery,
    delegationsQuery: DelegationsQuery,
    validatorsQuery: ValidatorsQuery,
    private val stakeService: GemStakeServiceInterface,
    getSession: GetSession,
    stateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : ViewModel() {

    private val assetId = stateHandle.get<String>(RouteArgument.AssetId.key)?.toAssetId()
        ?: error("Missing assetId")

    val assetInfo = getCurrentWalletId().flatMapLatest { walletId -> assetQuery(walletId.id, assetId) }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val providers = validatorsQuery(assetId, StakeProviderType.Earn)
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val delegations = session.filterNotNull()
        .flatMapLatest { current -> delegationsQuery(current.wallet.id, assetId, StakeProviderType.Earn) }

    private val loadState = MutableStateFlow<GemLoadState>(GemLoadState.Loading)

    val earnView = combine(providers, delegations, assetInfo, session, loadState) { providers, delegations, assetInfo, current, state ->
        val info = assetInfo ?: return@combine null
        stakeService.earnView(
            GemEarnInput(
                walletType = (current?.wallet?.type ?: WalletType.View).toGem(),
                asset = info.asset.toGem(),
                providers = providers.map { it.toGem() },
                delegations = delegations.map { it.toGem() },
                assetApr = info.metadata?.earnApr,
                price = info.price?.price,
                currency = (current?.currency ?: Currency.USD).toGem(),
                state = state,
            ),
        )
    }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val positions = earnView.map { view -> view?.positions.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    fun onPosition(delegation: Delegation, onOpenDetail: (String, String) -> Unit, onAmount: AmountTransactionAction, onConfirm: ConfirmTransactionAction) {
        val item = positions.value.firstOrNull { it.delegation.toPrimitives() == delegation } ?: return
        item.destination.open(delegation, onOpenDetail, onAmount, onConfirm)
    }

    val depositParams = earnView.map { view -> view?.depositProvider?.let { AmountParams.Earn.Deposit(assetId, it.id) } }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val sync = MutableStateFlow(true)

    val isSync = sync
        .flatMapLatest { isSync ->
            flow {
                if (!isSync) {
                    emit(false)
                    return@flow
                }
                assetInfo.filterNotNull().first()
                emit(true)
                val state = stakeService.refreshEarn(assetId.toIdentifier(), positions.value.isNotEmpty())
                (state as? GemLoadState.Error)?.let { Log.e(TAG, "earn sync failed", it.error) }
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

    private companion object {
        const val TAG = "Earn"
    }
}

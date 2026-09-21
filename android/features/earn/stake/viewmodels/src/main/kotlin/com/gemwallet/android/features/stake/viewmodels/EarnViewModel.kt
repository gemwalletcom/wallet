package com.gemwallet.android.features.stake.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.stake.cases.GetDelegations
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.list_item.ListItemModel
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletType
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
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemListRowTitle
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemServiceException
import uniffi.gemstone.GemStakeServiceInterface
import uniffi.gemstone.GemValidatorRow
import uniffi.gemstone.validatorRow
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class EarnViewModel @Inject constructor(
    getAssetInfo: GetAssetInfo,
    getDelegations: GetDelegations,
    getValidators: GetValidators,
    private val stakeService: GemStakeServiceInterface,
    getSession: GetSession,
    stateHandle: SavedStateHandle,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel() {

    private val assetId = stateHandle.get<String>(RouteArgument.AssetId.key)?.toAssetId()
        ?: error("Missing assetId")

    val assetInfo = getAssetInfo(assetId)
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val providers = getValidators(assetId, StakeProviderType.Earn)
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val positions = session.filterNotNull()
        .flatMapLatest { current -> getDelegations(current.wallet.id, assetId, StakeProviderType.Earn) }
        .map { delegations -> stakeService.positions(delegations.map { it.toGem() }).map { it.toPrimitives() } }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val validatorRows = positions
        .map { items -> items.associate { it.validator.id to validatorRow(it.validator.toGem()) } }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyMap<String, GemValidatorRow>())

    val aprRow: StateFlow<GemListRow> = combine(providers, assetInfo) { items, current ->
        stakeService.earnAprRow(items.map { it.toGem() }, current?.metadata?.earnApr)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, GemListRow.Text(GemListRowTitle.STAKE_APR, ""))

    val depositListItem = ListItemModel(title = context.getString(R.string.wallet_deposit))

    val depositParams = combine(providers, session) { items, current ->
        val provider = stakeService.earnActions((current?.wallet?.type ?: WalletType.View).toGem(), items.map { it.toGem() }).depositProvider ?: return@combine null
        AmountParams.Earn.Deposit(assetId, provider.id)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val sync = MutableStateFlow(true)
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

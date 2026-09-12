package com.gemwallet.android.features.stake.viewmodels

import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetAssetInfo
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.stake.cases.GetDelegations
import com.gemwallet.android.application.stake.cases.GetValidators
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AmountParams
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletType
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
import uniffi.gemstone.GemStakeServiceInterface
import uniffi.gemstone.GemValidatorRow
import java.math.BigInteger
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
) : ViewModel() {

    private val assetId = stateHandle.get<String>(RouteArgument.AssetId.key)?.toAssetId()
        ?: error("Missing assetId")

    val assetInfo = getAssetInfo(assetId)
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val session = getSession()
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val providers = getValidators(assetId, StakeProviderType.Earn)
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val positions = session.filterNotNull()
        .flatMapLatest { current -> getDelegations(current.wallet.id, assetId, StakeProviderType.Earn) }
        .map { delegations -> delegations.filter { it.base.balance > BigInteger.ZERO } }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val validatorRows = positions
        .map { items -> items.associate { it.validator.id to stakeService.validatorRow(it.validator.toGem()) } }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyMap<String, GemValidatorRow>())

    val apr = combine(providers, assetInfo) { items, current ->
        stakeService.earnApr(items.map { it.toGem() }, current?.metadata?.earnApr)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, 0.0)

    val depositParams = combine(providers, session) { items, current ->
        val provider = items.firstOrNull() ?: return@combine null
        if (current?.wallet?.type == WalletType.View) return@combine null
        AmountParams.Earn.Deposit(assetId, provider.id)
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

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
                runCatchingCancellable { stakeService.syncEarn(assetId.toIdentifier()) }
                    .onFailure { Log.e(TAG, "earn sync failed", it) }
                emit(false)
                sync.update { false }
            }
        }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    fun onRefresh() {
        sync.update { true }
    }

    private companion object {
        const val TAG = "Earn"
    }
}

package com.gemwallet.android.features.asset.viewmodels.details.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.application.assets.coordinators.GetChainAssetInfo
import com.gemwallet.android.application.assets.coordinators.SyncAssetInfo
import com.gemwallet.android.application.assets.coordinators.ToggleAssetPin
import com.gemwallet.android.application.pricealerts.coordinators.GetPriceAlerts
import com.gemwallet.android.application.pricealerts.coordinators.HasAssetPriceAlerts
import com.gemwallet.android.application.pricealerts.coordinators.PriceAlertsStateCoordinator
import com.gemwallet.android.application.pricealerts.coordinators.UpdatePriceAlerts
import com.gemwallet.android.application.session.coordinators.GetSession
import com.gemwallet.android.application.transactions.coordinators.GetTransactions
import com.gemwallet.android.application.transactions.coordinators.SyncAssetTransactions
import com.gemwallet.android.application.transactions.coordinators.TransactionsRequestFilter
import com.gemwallet.android.cases.banners.HasMultiSign
import com.gemwallet.android.cases.nodes.GetCurrentBlockExplorer
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.pricealerts.values.PriceAlertsStateEvent
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModelFactory
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Wallet
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import uniffi.gemstone.Explorer
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AssetDetailsViewModel @Inject constructor(
    getSession: GetSession,
    savedStateHandle: SavedStateHandle,
    private val getChainAssetInfo: GetChainAssetInfo,
    private val toggleAssetPin: ToggleAssetPin,
    private val enableAsset: EnableAsset,
    private val syncAssetInfo: SyncAssetInfo,
    private val getTransactions: GetTransactions,
    private val priceAlertsStateCoordinator: PriceAlertsStateCoordinator,
    private val hasAssetPriceAlerts: HasAssetPriceAlerts,
    private val updatePriceAlerts: UpdatePriceAlerts,
    private val getPriceAlerts: GetPriceAlerts,
    private val getCurrentBlockExplorer: GetCurrentBlockExplorer,
    private val hasMultiSign: HasMultiSign,
    private val syncAssetTransactions: SyncAssetTransactions,
) : ViewModel() {
    private var syncJob: Job? = null

    val session = getSession()

    val isRefreshing = MutableStateFlow(false)

    private val assetId = savedStateHandle.requireAssetId()

    private val chainAssetInfo = getChainAssetInfo(assetId)
        .onStart {
            val wallet = session.value?.wallet ?: return@onStart
            priceAlertsStateCoordinator.priceAlertState(PriceAlertsStateEvent.Request(assetId))
            syncAssetDetails(wallet, assetId, shouldRefreshPriceAlerts = true)
        }
        .filterNotNull()

    val isOperationEnabled = session.filterNotNull().flatMapLatest {
        hasMultiSign.hasMultiSign(it.wallet).mapLatest { !it }
    }
    .stateIn(viewModelScope, SharingStarted.Eagerly, true)

    private val model = chainAssetInfo.map { chainInfo ->
        val explorerName = getCurrentBlockExplorer.getCurrentBlockExplorer(chainInfo.assetInfo.asset.chain)
        Model(
            chainAssetInfo = chainInfo,
            explorerName = explorerName,
            updatedAt = System.currentTimeMillis()
        )
    }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val priceAlertEnabled = priceAlertsStateCoordinator.priceAlertState
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    val priceAlertsCount = getPriceAlerts(assetId)
        .map { it.size }
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0)

    val transactions = getTransactions.getTransactions(listOf(TransactionsRequestFilter.Asset(assetId)))
        .map { it.toImmutableList() }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    val uiModel = model.map { current ->
        current?.let { AssetInfoUIModelFactory.create(it.chainAssetInfo, it.explorerName) }
    }
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    fun refresh() {
        val wallet = session.value?.wallet ?: return
        syncAssetDetails(wallet, assetId, showLoading = true, shouldRefreshPriceAlerts = true)
    }

    private fun syncAssetDetails(
        wallet: Wallet,
        assetId: AssetId,
        showLoading: Boolean = false,
        shouldRefreshPriceAlerts: Boolean = false,
    ) {
        val previousJob = syncJob
        if (previousJob?.isActive == true) {
            if (showLoading) {
                return
            }
        }

        if (showLoading) {
            isRefreshing.value = true
        }

        if (shouldRefreshPriceAlerts) {
            refreshPriceAlertsIfNeeded(assetId)
        }

        syncJob = viewModelScope.launch(Dispatchers.IO) {
            if (previousJob?.isActive == true) {
                previousJob.cancelAndJoin()
            }

            try {
                refreshAssetDetails(wallet, assetId)
            } finally {
                if (showLoading) {
                    isRefreshing.value = false
                }
            }
        }
    }

    private suspend fun refreshAssetDetails(wallet: Wallet, assetId: AssetId) = coroutineScope {
        launch { syncAssetInfo.syncAssetInfo(assetId = assetId, wallet = wallet) }
        launch { syncAssetTransactions.syncAssetTransactions(assetId) }
    }

    private fun refreshPriceAlertsIfNeeded(assetId: AssetId) = viewModelScope.launch(Dispatchers.IO) {
        if (hasAssetPriceAlerts(assetId)) {
            runCatching { updatePriceAlerts.update(assetId) }
        }
    }

    fun enablePriceAlert(assetId: AssetId) = viewModelScope.launch {
        val event = when (priceAlertEnabled.value) {
            is PriceAlertsStateEvent.Disable -> PriceAlertsStateEvent.Enable(assetId)
            is PriceAlertsStateEvent.Enable -> PriceAlertsStateEvent.Disable(assetId)
            else -> return@launch
        }
        priceAlertsStateCoordinator.priceAlertState(event)
    }

    fun pin() = viewModelScope.launch(Dispatchers.IO) {
        val wallet = session.value?.wallet ?: return@launch
        val assetInfo = model.value?.chainAssetInfo?.assetInfo ?: return@launch
        val assetId = assetInfo.id()
        wallet.getAccount(assetId) ?: return@launch
        toggleAssetPin(assetId)
    }

    fun add() = viewModelScope.launch(Dispatchers.IO) {
        val session = session.value ?: return@launch
        val assetInfo = model.value?.chainAssetInfo?.assetInfo ?: return@launch

        add(session.wallet, assetInfo.id())
    }

    private suspend fun add(wallet: Wallet, assetId: AssetId) {
        wallet.getAccount(assetId) ?: return
        enableAsset(wallet.id, assetId)
    }

    private data class Model(
        val chainAssetInfo: ChainAssetInfo,
        val updatedAt: Long,
        val explorerName: String,
    )
}

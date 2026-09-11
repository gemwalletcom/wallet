package com.gemwallet.android.features.asset.viewmodels.details.viewmodels

import android.util.Log
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.serviceMessage
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.ext.toGem
import uniffi.gemstone.GemAssetDetailsInput
import uniffi.gemstone.GemAssetDetailsServiceInterface
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.cases.GetChainAssetInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.application.banner.cases.GetActiveBanners
import com.gemwallet.android.application.pricealerts.cases.GetPriceAlerts
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.model.Session
import com.gemwallet.android.model.toGem
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModelFactory
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.BannerEvent
import com.wallet.core.primitives.PriceAlert
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.launch
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import javax.inject.Inject

@OptIn(ExperimentalCoroutinesApi::class)
@HiltViewModel
class AssetDetailsViewModel @Inject constructor(
    getSession: GetSession,
    savedStateHandle: SavedStateHandle,
    private val getChainAssetInfo: GetChainAssetInfo,
    private val getWalletAssets: GetWalletAssets,
    private val getTransactions: GetTransactions,
    private val assetDetailsService: GemAssetDetailsServiceInterface,
    private val getActiveBanners: GetActiveBanners,
    private val getPriceAlerts: GetPriceAlerts,
    private val assetInfoUIModelFactory: AssetInfoUIModelFactory,
) : ViewModel() {
    private var syncJob: Job? = null

    val session = getSession()

    val isRefreshing = MutableStateFlow(false)

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    private val assetId = savedStateHandle.requireAssetId()

    private val chainAssetInfo = getChainAssetInfo(assetId)
        .onStart { restartAssetSync() }
        .filterNotNull()
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, storedChainAssetInfo())

    private fun storedChainAssetInfo(): ChainAssetInfo? {
        val stored = getWalletAssets().value
        val assetInfo = stored.firstOrNull { it.asset.id == assetId } ?: return null
        val feeInfo = stored.firstOrNull { it.asset.id == AssetId(assetId.chain) } ?: return null
        return ChainAssetInfo(assetInfo, feeInfo)
    }

    val transactions = getTransactions.getTransactions(listOf(TransactionsRequestFilter.Asset(assetId)))
        .map { it.toImmutableList() }
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, emptyList())

    private val bannerEvents = chainAssetInfo.filterNotNull()
        .map { it.assetInfo.asset }
        .distinctUntilChanged()
        .flatMapLatest { getActiveBanners(it, isGlobal = false) }
        .map { banners -> banners.map { it.event } }

    private val priceAlerts = getPriceAlerts.assetPriceAlerts(assetId)

    val uiModel = combine(chainAssetInfo, session, bannerEvents, priceAlerts, ::uiModel)
        .flowOn(Dispatchers.IO)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private fun uiModel(
        chainInfo: ChainAssetInfo?,
        session: Session?,
        bannerEvents: List<BannerEvent>,
        priceAlerts: List<PriceAlert>,
    ): AssetInfoUIModel? {
        val wallet = session?.wallet ?: return null
        val assetInfo = chainInfo?.assetInfo ?: return null
        val details = assetDetailsService.details(
            GemAssetDetailsInput(
                walletType = wallet.type.toGem(),
                asset = assetInfo.asset.toGem(),
                ownerAddress = assetInfo.owner?.address,
                metadata = assetInfo.metadata.toGem(),
                balance = assetInfo.balance.toGem(),
                price = assetInfo.price?.price?.price,
                bannerEvents = bannerEvents.map { event -> event.toGem() },
                priceAlerts = priceAlerts.map { alert -> alert.toGem() },
            )
        )
        return assetInfoUIModelFactory.create(chainAssetInfo = chainInfo, details = details)
    }

    fun refresh() {
        if (syncJob?.isActive == true) {
            return
        }

        isRefreshing.value = true
        syncPriceAlerts()
        syncJob = viewModelScope.launch(Dispatchers.IO) {
            try {
                syncAssetDetails()
            } finally {
                isRefreshing.value = false
            }
        }
    }

    private fun restartAssetSync() {
        val previousJob = syncJob

        syncPriceAlerts()
        syncJob = viewModelScope.launch(Dispatchers.IO) {
            previousJob?.cancelAndJoin()
            syncAssetDetails()
        }
    }

    private fun syncPriceAlerts() = viewModelScope.launch(Dispatchers.IO) {
        runCatchingCancellable { assetDetailsService.syncPriceAlerts(assetId.toIdentifier()) }
            .onFailure { Log.e(TAG, "price alerts sync failed for ${assetId.toIdentifier()}", it) }
    }

    private suspend fun syncAssetDetails() {
        assetDetailsService
            .refresh(assetId.toIdentifier())
            .forEach { Log.e(TAG, "asset refresh ${it.step} failed: ${it.message}") }
    }

    fun pin() = viewModelScope.launch(Dispatchers.IO) {
        val assetInfo = chainAssetInfo.value?.assetInfo ?: return@launch
        assetDetailsService.setAssetPinned(assetInfo.id().toIdentifier(), !assetInfo.metadata.isPinned)
    }

    fun add() = viewModelScope.launch(Dispatchers.IO) {
        val assetInfo = chainAssetInfo.value?.assetInfo ?: return@launch
        assetDetailsService.setAssetsEnabled(listOf(assetInfo.id().toIdentifier()), true)
    }

    fun togglePriceAlert(assetId: AssetId) = viewModelScope.launch(Dispatchers.IO) {
        val enabled = uiModel.value?.detailsState?.priceAlertEnabled ?: return@launch
        runCatchingCancellable { assetDetailsService.setPriceAlert(assetId.toIdentifier(), !enabled) }
            .onFailure { errorState.value = it.serviceMessage() }
    }

    fun clearError() = errorState.update { null }

    private companion object {
        const val TAG = "AssetDetails"
    }
}

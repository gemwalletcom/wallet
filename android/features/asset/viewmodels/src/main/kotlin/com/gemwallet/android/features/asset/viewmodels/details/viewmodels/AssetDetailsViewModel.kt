package com.gemwallet.android.features.asset.viewmodels.details.viewmodels

import android.content.Context
import android.util.Log
import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.IoDispatcher
import com.gemwallet.android.application.assets.cases.GetChainAssetInfo
import com.gemwallet.android.application.assets.cases.GetWalletAssets
import com.gemwallet.android.application.session.cases.GetSession
import com.gemwallet.android.application.transactions.cases.GetTransactions
import com.gemwallet.android.application.transactions.cases.TransactionsRequestFilter
import com.gemwallet.android.data.services.gemstone.config.UserConfig
import com.gemwallet.android.data.services.gemstone.connection.ConnectionStatusObserver
import com.gemwallet.android.data.services.store.queries.BannersQuery
import com.gemwallet.android.data.services.store.queries.PriceAlertsQuery
import com.gemwallet.android.ext.errorText
import com.gemwallet.android.ext.runCatchingCancellable
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModelFactory
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.model.Session
import com.gemwallet.android.model.toGem
import com.gemwallet.android.ui.R
import com.gemwallet.android.ui.components.screen.assetAddedToast
import com.gemwallet.android.ui.components.screen.assetPinnedToast
import com.gemwallet.android.ui.localization.text
import com.gemwallet.android.ui.localization.toastRes
import com.gemwallet.android.ui.models.ToastEmitter
import com.gemwallet.android.ui.models.ToastEmitterImpl
import com.gemwallet.android.ui.models.ToastMessage
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.Banner
import com.wallet.core.primitives.PriceAlert
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.collections.immutable.toImmutableList
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancelAndJoin
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
import kotlinx.coroutines.launch
import uniffi.gemstone.GemAssetDetailsInput
import uniffi.gemstone.GemAssetDetailsServiceInterface
import uniffi.gemstone.GemBannerKey
import uniffi.gemstone.GemListRow
import uniffi.gemstone.GemLoadState
import uniffi.gemstone.GemPriceAlertToggle
import uniffi.gemstone.GemRefreshKind
import uniffi.gemstone.loadError
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
    private val bannersQuery: BannersQuery,
    private val priceAlertsQuery: PriceAlertsQuery,
    private val assetInfoUIModelFactory: AssetInfoUIModelFactory,
    private val userConfig: UserConfig,
    private val connectionStatusObserver: ConnectionStatusObserver,
    @param:IoDispatcher private val ioDispatcher: CoroutineDispatcher,
    @param:ApplicationContext private val context: Context,
) : ViewModel(),
    ToastEmitter by ToastEmitterImpl() {

    val refreshIntervalMillis: StateFlow<Long> = connectionStatusObserver.refreshIntervalMillis(GemRefreshKind.WALLET)
        .stateIn(viewModelScope, SharingStarted.Eagerly, 0L)

    private var syncJob: Job? = null

    val session = getSession()

    val isRefreshing = MutableStateFlow(false)

    private val errorState = MutableStateFlow<String?>(null)
    val error: StateFlow<String?> = errorState.asStateFlow()

    private val assetId = savedStateHandle.requireAssetId()

    private val transactionFilters = listOf(TransactionsRequestFilter.Asset(assetId))

    val transactions = getTransactions.getTransactions(transactionFilters)
        .map { it.toImmutableList() }
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, getTransactions.stored(transactionFilters).toImmutableList())

    private val transactionsState = MutableStateFlow<GemLoadState>(GemLoadState.Loading)

    private val chainAssetInfo = getChainAssetInfo(assetId)
        .onStart { restartAssetSync() }
        .filterNotNull()
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, storedChainAssetInfo())

    private fun storedChainAssetInfo(): ChainAssetInfo? {
        val stored = getWalletAssets().value
        val assetInfo = stored.firstOrNull { it.asset.id == assetId } ?: return null
        val feeInfo = stored.firstOrNull { it.asset.id == AssetId(assetId.chain) } ?: return null
        return ChainAssetInfo(assetInfo, feeInfo)
    }

    val transactionsErrorRow: StateFlow<GemListRow?> = combine(transactionsState, transactions) { state, items ->
        loadError(state, items.isNotEmpty())?.let { GemListRow.Error(it) }
    }.stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private val banners = chainAssetInfo.filterNotNull()
        .map { it.assetInfo.asset }
        .distinctUntilChanged()
        .flatMapLatest { asset ->
            session.flatMapLatest { bannersQuery(it?.wallet?.id?.id, asset.id) }
        }

    private val priceAlerts = priceAlertsQuery(assetId).map { alerts -> alerts.map { it.priceAlert } }

    val uiModel = combine(chainAssetInfo, session, banners, priceAlerts, ::uiModel)
        .flowOn(ioDispatcher)
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private fun uiModel(chainInfo: ChainAssetInfo?, session: Session?, banners: List<Banner>, priceAlerts: List<PriceAlert>): AssetInfoUIModel? {
        session ?: return null
        val wallet = session.wallet
        val assetInfo = chainInfo?.assetInfo ?: return null
        val details = assetDetailsService.details(
            GemAssetDetailsInput(
                wallet = wallet.toGem(),
                asset = assetInfo.asset.toGem(),
                ownerAddress = assetInfo.owner?.address,
                metadata = assetInfo.metadata.toGem(),
                balance = assetInfo.balance.toGem(),
                price = assetInfo.price?.price?.price,
                priceChangePercentage24h = assetInfo.price?.price?.priceChangePercentage24h,
                currency = session.currency.toGem(),
                banners = banners.map { banner -> banner.toGem() },
                priceAlerts = priceAlerts.map { alert -> alert.toGem() },
                feeBalanceMetadata = chainInfo.feeAssetInfo.balance.metadata?.toGem(),
            ),
        )
        return assetInfoUIModelFactory.create(chainAssetInfo = chainInfo, details = details)
    }

    fun refresh() {
        if (syncJob?.isActive == true) {
            return
        }

        isRefreshing.value = true
        syncJob = viewModelScope.launch(ioDispatcher) {
            try {
                syncAssetDetails()
            } finally {
                isRefreshing.value = false
            }
        }
    }

    private fun restartAssetSync() {
        val previousJob = syncJob

        syncJob = viewModelScope.launch(ioDispatcher) {
            previousJob?.cancelAndJoin()
            syncAssetDetails()
        }
    }

    private suspend fun syncAssetDetails() {
        val refresh = assetDetailsService.refresh(assetId.toIdentifier(), transactions.value.isNotEmpty())
        transactionsState.value = refresh.transactions
        refresh.failures.forEach { Log.e(TAG, "asset refresh ${it.step} failed: ${it.message}") }
    }

    fun pin() = viewModelScope.launch(ioDispatcher) {
        val assetInfo = chainAssetInfo.value?.assetInfo ?: return@launch
        runCatchingCancellable { assetDetailsService.setAssetPinned(assetInfo.id().toIdentifier(), !assetInfo.metadata.isPinned) }
            .onSuccess { emitToast(assetPinnedToast(context, assetInfo.asset.name, !assetInfo.metadata.isPinned)) }
            .onFailure { Log.e(TAG, "pinning ${assetInfo.id().toIdentifier()} failed", it) }
    }

    fun add() = viewModelScope.launch(ioDispatcher) {
        val assetInfo = chainAssetInfo.value?.assetInfo ?: return@launch
        runCatchingCancellable { assetDetailsService.setAssetsEnabled(listOf(assetInfo.id().toIdentifier()), true) }
            .onSuccess { emitToast(assetAddedToast(context)) }
            .onFailure { emitToast(ToastMessage(it.errorText().text(context), R.drawable.ic_error)) }
    }

    fun togglePriceAlert(assetId: AssetId) = viewModelScope.launch(ioDispatcher) {
        val current = uiModel.value?.details?.state?.priceAlert ?: return@launch
        val name = chainAssetInfo.value?.assetInfo?.asset?.name.orEmpty()
        runCatchingCancellable { assetDetailsService.setPriceAlert(assetId.toIdentifier(), current.toggled() == GemPriceAlertToggle.ENABLED) }
            .onSuccess { emitToast(ToastMessage(context.getString(current.toastRes(), name), R.drawable.ic_notifications)) }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun closeBanner(key: GemBannerKey) = viewModelScope.launch(ioDispatcher) {
        runCatchingCancellable { assetDetailsService.closeBanner(key) }
            .onFailure { errorState.value = it.errorText().text(context) }
    }

    fun enablePerpetuals() {
        userConfig.setPerpetualEnabled(true)
    }

    fun clearError() = errorState.update { null }

    private companion object {
        const val TAG = "AssetDetails"
    }
}

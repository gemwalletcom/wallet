package com.gemwallet.android.features.asset.viewmodels.details.viewmodels

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.gemwallet.android.application.assets.coordinators.EnableAsset
import com.gemwallet.android.application.assets.coordinators.GetAssetTokenInfo
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
import com.gemwallet.android.domains.asset.getIconUrl
import com.gemwallet.android.domains.percentage.PercentageFormatterStyle
import com.gemwallet.android.domains.percentage.formatAsPercentage
import com.gemwallet.android.domains.price.toValueDirection
import com.gemwallet.android.domains.pricealerts.values.PriceAlertsStateEvent
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.isStaked
import com.gemwallet.android.ext.type
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.availableFormatted
import com.gemwallet.android.model.format
import com.gemwallet.android.model.getStackedAmount
import com.gemwallet.android.model.reservedFormatted
import com.gemwallet.android.model.totalFormatted
import com.gemwallet.android.model.totalStakeFormatted
import com.gemwallet.android.features.asset.viewmodels.details.models.AssetInfoUIModel
import com.gemwallet.android.ui.models.navigation.requireAssetId
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.StakeChain
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
    private val getAssetTokenInfo: GetAssetTokenInfo,
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

    private val assetInfo = getAssetTokenInfo(assetId)
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

    private val model = assetInfo
        .map {
            val explorerName = getCurrentBlockExplorer.getCurrentBlockExplorer(it.asset.chain)
            Model(
                assetInfo = it,
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

    val uiModel = model.map { it?.toUIState() }
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
        val assetInfo = model.value?.assetInfo ?: return@launch
        val assetId = assetInfo.id()
        wallet.getAccount(assetId) ?: return@launch
        toggleAssetPin(assetId)
    }

    fun add() = viewModelScope.launch(Dispatchers.IO) {
        val session = session.value ?: return@launch
        val assetInfo = model.value?.assetInfo ?: return@launch

        add(session.wallet, assetInfo.id())
    }

    private suspend fun add(wallet: Wallet, assetId: AssetId) {
        wallet.getAccount(assetId) ?: return
        enableAsset(wallet.id, assetId)
    }

    private data class Model(
        val assetInfo: AssetInfo,
        val updatedAt: Long,
        val explorerName: String,
    ) {
        fun toUIState(): AssetInfoUIModel {
            val assetInfo = assetInfo
            val price = assetInfo.price?.price?.price ?: 0.0
            val currency = assetInfo.price?.currency ?: Currency.USD
            val asset = assetInfo.asset
            val balances = assetInfo.balance
            val total = balances.totalAmount
            val fiatTotal = if (balances.fiatTotalAmount == 0.0) "" else currency.format(balances.fiatTotalAmount, dynamicPlace = true)
            val stakeBalance = balances.balanceAmount.getStackedAmount()

            return AssetInfoUIModel(
                assetInfo = assetInfo,
                name = if (asset.type == AssetType.NATIVE) { // TODO: ????
                    asset.id.chain.asset().name
                } else {
                    asset.name
                },
                iconUrl = asset.id.getIconUrl(),
                priceValue = if (price == 0.0) "" else currency.format(price, dynamicPlace = true),
                priceDayChanges = assetInfo.price?.price?.priceChangePercentage24h.formatAsPercentage(),
                priceChangedType = assetInfo.price?.price?.priceChangePercentage24h.toValueDirection(),
                tokenType = asset.type,
                isBuyEnabled = assetInfo.metadata?.isBuyEnabled == true,
                isSwapEnabled = assetInfo.metadata?.isSwapEnabled == true,
                explorerName = explorerName, //  TODO: Out to separate state and model
                explorerAddressUrl = assetInfo.owner?.address?.let {//  TODO: Out to separate state
                    Explorer(asset.chain.string).getAddressUrl(explorerName,  it)
                },
                explorerTokenUrl = asset.id.tokenId?.let {
                    Explorer(asset.chain.string).getTokenUrl(explorerName, it)
                },
                accountInfoUIModel = AssetInfoUIModel.AccountInfoUIModel(
                    walletType = assetInfo.walletType,
                    totalBalance = balances.totalFormatted(),
                    totalFiat = fiatTotal,
                    owner = assetInfo.owner?.address ?: "",
                    balanceMetadata = assetInfo.balance.metadata,
                    hasBalanceDetails = StakeChain.isStaked(asset.id.chain) || balances.balanceAmount.reserved != 0.0,
                    available = if (balances.balanceAmount.available != total) {
                        balances.availableFormatted()
                    } else {
                        ""
                    },
                    stake = if (asset.id.type() == AssetSubtype.NATIVE && StakeChain.isStaked(asset.id.chain)) {
                        if (stakeBalance == 0.0) {
                            "APR ${(assetInfo.stakeApr ?: 0.0).formatAsPercentage(style = PercentageFormatterStyle.PercentSignLess)}"
                        } else {
                            balances.totalStakeFormatted()
                        }
                    } else {
                        ""
                    },
                    reserved = if (balances.balanceAmount.reserved != 0.0) {
                        balances.reservedFormatted()
                    } else {
                        ""
                    },
                ),
            )
        }
    }
}

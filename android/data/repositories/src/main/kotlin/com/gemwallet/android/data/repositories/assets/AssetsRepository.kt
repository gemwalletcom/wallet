package com.gemwallet.android.data.repositories.assets

import android.util.Log
import com.gemwallet.android.application.transactions.coordinators.GetChangedTransactions
import com.gemwallet.android.blockchain.operators.GetAsset
import com.gemwallet.android.blockchain.services.BalancesService
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.stream.StreamSubscriptionService
import com.gemwallet.android.data.repositories.tokens.toPriorityQuery
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.AssetsPriorityDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.DbAsset
import com.gemwallet.android.data.service.store.database.entities.DbAssetBasicUpdate
import com.gemwallet.android.data.service.store.database.entities.DbRecentActivity
import com.gemwallet.android.data.service.store.database.entities.toAssetInfoModel
import com.gemwallet.android.data.service.store.database.entities.toAssetLinkRecord
import com.gemwallet.android.data.service.store.database.entities.toAssetLinksModel
import com.gemwallet.android.data.service.store.database.entities.toMarketRecord
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toPriceRecord
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.domains.asset.calculateAvailabilityChanges
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.defaultBasic
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.available
import com.gemwallet.android.ext.getAssociatedAssetIds
import com.gemwallet.android.ext.swapSupport
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.model.RecentAsset
import com.gemwallet.android.model.RecentAssetsRequest
import com.gemwallet.android.model.RecentType
import com.gemwallet.android.model.TransactionExtended
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetFull
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatRate
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

private const val TAG = "AssetsRepository"

@OptIn(ExperimentalCoroutinesApi::class)
@Singleton
class AssetsRepository @Inject constructor(
    private val assetsDao: AssetsDao,
    private val assetsPriorityDao: AssetsPriorityDao,
    private val balancesDao: BalancesDao,
    private val pricesDao: PricesDao,
    private val sessionRepository: SessionRepository,
    private val balancesService: BalancesService,
    getChangedTransactions: GetChangedTransactions,
    private val searchTokensCase: SearchTokensCase,
    private val streamSubscriptionService: StreamSubscriptionService,
    private val updateBalances: UpdateBalances = UpdateBalances(balancesDao, balancesService),
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetAsset {


    init {
        scope.launch(Dispatchers.IO) {
            getChangedTransactions.getChangedTransactions().collect {
                onTransactions(it)
            }
        }
        scope.launch(Dispatchers.IO) {
            sessionRepository.session().collectLatest {
                changeCurrency(it?.currency ?: return@collectLatest)
            }
        }
        scope.launch(Dispatchers.IO) {
            syncSwapSupportChains()
        }
    }

    suspend fun sync() {
        getAssetsInfo().firstOrNull()?.updateBalances()?.awaitAll()
    }

    suspend fun updateAssetMetadata(assetFull: AssetFull) = withContext(Dispatchers.IO) {
        val assetId = assetFull.asset.id
        val assetIdIdentifier = assetId.toIdentifier()
        val currency = sessionRepository.getCurrentCurrency()
        val rate = getCurrencyRate(currency).firstOrNull() ?: when (currency) {
            Currency.USD -> FiatRate(Currency.USD.string, 1.0)
            else -> null
        }
        val record = assetFull.toRecord().copy(
            updatedAt = System.currentTimeMillis(),
        )
        val linkRecords = assetFull.links.toAssetLinkRecord(assetId)
        assetsDao.update(record)
        runCatching { assetsDao.addLinks(linkRecords) }
            .onFailure { Log.e(TAG, "Failed to update asset links for $assetIdIdentifier", it) }
        rate?.let { fiatRate ->
            val currentPrice = pricesDao.getByAssets(listOf(assetIdIdentifier)).firstOrNull()
            val priceRecord = assetFull.toPriceRecord(fiatRate)
            if (priceRecord != null && (currentPrice == null || currentPrice.currency != priceRecord.currency || currentPrice.value == null)) {
                pricesDao.insert(priceRecord)
            }
            assetFull.toMarketRecord(fiatRate.rate)?.let { assetsDao.setMarket(it) }
        }
    }

    suspend fun updateAssetMarket(assetId: AssetId, market: AssetMarket, currency: Currency) = withContext(Dispatchers.IO) {
        val rate = getCurrencyRate(currency).firstOrNull()?.rate ?: return@withContext
        assetsDao.setMarket(market.toRecord(assetId, rate))
    }

    /**
     *  Create assets for new wallet(import or create wallet)
     *  */
    suspend fun createAssets(wallet: Wallet) {
        val assetIds = mutableListOf<AssetId>()
        wallet.accounts.forEach { account ->
            val asset = account.chain.asset()
            val isVisible = account.isVisibleByDefault(wallet.type)
            insertLocalAsset(wallet.id, asset, isVisible)
            if (isVisible) assetIds.add(asset.id)
        }
        if (assetIds.isNotEmpty()) {
            streamSubscriptionService.addAssetIds(assetIds)
        }
    }

    suspend fun getNativeAssets(wallet: Wallet): List<Asset> = withContext(Dispatchers.IO) {
        assetsDao.getNativeWalletAssets(wallet.id)
            .firstOrNull()
            ?.toDTO()
            ?: emptyList()
    }

    override suspend fun getAsset(assetId: AssetId): Asset? = withContext(Dispatchers.IO) {
        getAssetInfo(assetId).firstOrNull()?.asset
    }

    suspend fun hasAssets(assetIds: List<AssetId>): Set<AssetId> = withContext(Dispatchers.IO) {
        if (assetIds.isEmpty()) {
            return@withContext emptySet()
        }
        assetsDao.getAssetIds(assetIds.map { it.toIdentifier() })
            .mapNotNull { it.toAssetId() }
            .toSet()
    }

    suspend fun hasWalletAssets(walletId: String, assetIds: List<AssetId>): Set<AssetId> = withContext(Dispatchers.IO) {
        if (assetIds.isEmpty()) {
            return@withContext emptySet()
        }
        assetsDao.getWalletAssetIds(walletId, assetIds.map { it.toIdentifier() })
            .mapNotNull { it.toAssetId() }
            .toSet()
    }

    fun getAssetsInfo(): Flow<List<AssetInfo>> = assetsDao.getAssetsInfo()
        .toAssetInfoModel()

    fun getAssetsInfo(assetsId: List<AssetId>): Flow<List<AssetInfo>> = assetsDao
        .getAssetsInfo(assetsId.map { it.toIdentifier() })
        .toAssetInfoModel()
        .flowOn(Dispatchers.IO)


    fun getAssetInfo(assetId: AssetId): Flow<AssetInfo?> {
        return assetsDao.getAssetInfo(assetId.toIdentifier(), assetId.chain)
            .map { it?.toDTO() }
            .flowOn(Dispatchers.IO)
    }

    fun asset(assetId: AssetId): Flow<Asset?> {
        return assetsDao.getAsset(assetId.toIdentifier())
            .map { it?.toDTO() }
            .flowOn(Dispatchers.IO)
    }

    suspend fun getToken(assetId: AssetId): Flow<Asset?> = withContext(Dispatchers.IO) {
        assetsDao.getTokenInfo(assetId.toIdentifier(), assetId.chain).map { it?.toDTO()?.asset }
    }

    fun getTokenInfo(assetId: AssetId): Flow<AssetInfo?> {
        return assetsDao.getAssetInfo(assetId.toIdentifier(), assetId.chain).flatMapLatest { assetInfo ->
            if (assetInfo == null) {
                assetsDao.getTokenInfo(assetId.toIdentifier(), assetId.chain).map { it?.toDTO() }
            } else {
                flow { emit(assetInfo.toDTO()) }
            }
        }
        .flowOn(Dispatchers.IO)
    }

    fun getTokensInfo(assetsId: List<String>): Flow<List<AssetInfo>> {
        return assetsDao.getAssetsInfoByAllWallets(assetsId).toAssetInfoModel()
    }

    suspend fun getWidgetTokens(currency: Currency): List<AssetInfo> = withContext(Dispatchers.IO) {
        val widgetAssetIds = listOf(AssetId(Chain.Bitcoin), AssetId(Chain.Ethereum), AssetId(Chain.Solana))

        searchTokensCase.search(widgetAssetIds, currency)
        getTokensInfo(widgetAssetIds.map { it.toIdentifier() }).firstOrNull() ?: emptyList()
    }

    suspend fun searchToken(assetId: AssetId, currency: Currency): Boolean {
        return searchTokensCase.search(assetId, currency)
    }

    fun search(query: String, tags: List<AssetTag>, byAllWallets: Boolean): Flow<List<AssetInfo>> {
        val query = tags.toPriorityQuery(query)
        return if (byAllWallets) {
            assetsPriorityDao.hasPriorities(query).map { it > 0 }.flatMapLatest {
                if (it) {
                    assetsDao.searchByAllWalletsWithPriority(query)
                } else {
                    assetsDao.searchByAllWallets(query)
                }
            }
        } else {
            assetsPriorityDao.hasPriorities(query).map { it > 0 }.flatMapLatest {
                if (it) {
                    assetsDao.searchWithPriority(query)
                } else {
                    assetsDao.search(query)
                }
            }
        }
        .toAssetInfoModel()
    }

    fun swapSearch(wallet: Wallet, query: String, byChains: List<Chain>, byAssets: List<AssetId>, tags: List<AssetTag>): Flow<List<AssetInfo>> {
        val query = tags.toPriorityQuery(query)
        val walletChains = wallet.accounts.map { it.chain }
        val includeChains = byChains.filter { walletChains.contains(it) }
        val includeAssetIds = byAssets.filter { walletChains.contains(it.chain) }
        return assetsPriorityDao.hasPriorities(query).map { it > 0 }.flatMapLatest { hasPriority ->
                if (hasPriority) {
                    assetsDao.swapSearchWithPriority(query, includeChains, includeAssetIds.map { it.toIdentifier() })
                } else {
                    assetsDao.swapSearch(query, includeChains, includeAssetIds.map { it.toIdentifier() })
                }
            }
            .toAssetInfoModel()
            .map { assets ->
                assets.filter { asset ->
                    asset.metadata?.isEnabled == true
                }
            }
    }

    /**
     * Check and add new coins and active tokens
     * */
    fun invalidateDefault(wallet: Wallet) = scope.launch(Dispatchers.IO) {
        val assets = getNativeAssets(wallet).associateBy( { it.id.toIdentifier() }, { it })

        wallet.accounts.map { account ->
            val asset = account.chain.asset()
            async {
                if (assets[account.chain.string] == null) {
                    add(wallet.id, asset, false)
                    val balances = updateBalances.updateBalances(wallet.id, account, emptyList()).firstOrNull()
                    if ((balances?.totalAmount ?: 0.0) > 0.0) {
                        setVisibility(wallet.id, asset.id, true)
                    }
                }
            }
        }.awaitAll()
    }

    suspend fun switchVisibility(
        walletId: String,
        assetId: AssetId,
        visibility: Boolean,
    ) = withContext(Dispatchers.IO) {
        val assetInfo = getAssetInfo(assetId).firstOrNull()
        val isCurrentWalletAsset = assetInfo?.walletId == walletId
        val isVisible = assetInfo?.metadata?.isBalanceEnabled == true

        if (!isCurrentWalletAsset) {
            if (!visibility) {
                return@withContext
            }
            linkAssetToWallet(walletId, assetId, true)
            updateBalances(assetId)
            return@withContext
        }
        if (isVisible == visibility) {
            return@withContext
        }
        setVisibility(walletId, assetId, visibility)
        if (visibility) {
            updateBalances(assetId)
        }
    }

    suspend fun togglePin(walletId: String, assetId: AssetId) = withContext(Dispatchers.IO) {
        assetsDao.toggleWalletAssetPin(walletId, assetId.toIdentifier())
    }

    suspend fun updateBalances(vararg tokens: AssetId) {
        getAssetsInfo(tokens.toList()).firstOrNull()?.updateBalances()?.awaitAll()
    }

    suspend fun add(walletId: String, asset: Asset, visible: Boolean) {
        insertLocalAsset(walletId, asset, visible)
        if (visible) {
            streamSubscriptionService.addAssetIds(listOf(asset.id))
        }
    }

    suspend fun add(walletId: String, asset: AssetBasic, visible: Boolean) {
        insertAssetRecord(
            walletId = walletId,
            assetId = asset.asset.id,
            record = asset.toRecord(),
            visible = visible,
        )
        if (visible) {
            streamSubscriptionService.addAssetIds(listOf(asset.asset.id))
        }
    }

    suspend fun add(assets: List<AssetBasic>) = withContext(Dispatchers.IO) {
        if (assets.isEmpty()) {
            return@withContext
        }
        runCatching { assetsDao.insert(assets.map { it.toRecord() }) }
            .onFailure { Log.e(TAG, "Failed to insert ${assets.size} assets", it) }
    }

    suspend fun linkAssetToWallet(
        walletId: String,
        assetId: AssetId,
        visible: Boolean,
    ) = withContext(Dispatchers.IO) {
        assetsDao.setWalletAssetVisibility(walletId, assetId.toIdentifier(), visible)
    }

    suspend fun add(walletId: String, assets: List<Asset>, visible: Boolean) {
        assets.forEach { asset ->
            insertLocalAsset(walletId, asset, visible)
        }
        if (visible && assets.isNotEmpty()) {
            streamSubscriptionService.addAssetIds(assets.map { it.id })
        }
    }

    suspend fun updateNativeAssetRanks() = withContext(Dispatchers.IO) {
        for (chain in Chain.available()) {
            val assetBasic = chain.asset().defaultBasic
            runCatching { assetsDao.updateAssetRank(assetBasic.asset.id.toIdentifier(), assetBasic.score.rank) }
                .onFailure { Log.e(TAG, "Failed to update native asset rank for ${assetBasic.asset.id}", it) }
        }
    }

    private suspend fun insertLocalAsset(walletId: String, asset: Asset, visible: Boolean) {
        val assetBasic = asset.defaultBasic
        val assetId = asset.id
        insertAssetRecord(
            walletId = walletId,
            assetId = assetId,
            record = assetBasic.toRecord(),
            visible = visible,
        )
    }

    private suspend fun insertAssetRecord(
        walletId: String,
        assetId: AssetId,
        record: DbAsset,
        visible: Boolean,
    ) {
        val assetIdIdentifier = assetId.toIdentifier()
        // REPLACE would cascade-delete balances/accounts; insert-or-update keeps the asset row stable.
        assetsDao.insert(record)
        assetsDao.updateBasicAsset(record.toBasicUpdateRecord())
        assetsDao.setWalletAssetVisibility(walletId, assetIdIdentifier, visible)
    }

    suspend fun updateBuyAvailable(assets: List<String>) {
        syncAvailability(
            currentEnabledAssetIds = assetsDao.getBuyAvailableAssetIds(),
            targetEnabledAssetIds = assets,
            setAvailability = assetsDao::setBuyAvailable,
        )
    }

    suspend fun updateSellAvailable(assets: List<String>) {
        syncAvailability(
            currentEnabledAssetIds = assetsDao.getSellAvailableAssetIds(),
            targetEnabledAssetIds = assets,
            setAvailability = assetsDao::setSellAvailable,
        )
    }

    private fun onTransactions(txs: List<TransactionExtended>) = scope.launch {
        txs.map { txEx ->
            async {
                getAssetsInfo(txEx.transaction.getAssociatedAssetIds())
                    .firstOrNull()
                    ?.updateBalances()
            }
        }.awaitAll()
    }

    fun getAssetLinks(id: AssetId): Flow<List<AssetLink>> {
        return assetsDao.getAssetLinks(id.toIdentifier())
            .toAssetLinksModel()
            .flowOn(Dispatchers.IO)
    }

    fun getAssetMarket(id: AssetId): Flow<AssetMarket?> {
        return assetsDao.getAssetMarket(id.toIdentifier())
            .map { it?.toDTO() }
            .flowOn(Dispatchers.IO)
    }

    private fun Account.isVisibleByDefault(type: WalletType): Boolean {
        return visibleByDefault.contains(chain) || type != WalletType.Multicoin
    }

    private suspend fun setVisibility(walletId: String, assetId: AssetId, visibility: Boolean) = withContext(Dispatchers.IO) {
        assetsDao.setWalletAssetVisibility(walletId, assetId.toIdentifier(), visibility)

        if (visibility) {
            streamSubscriptionService.addAssetIds(listOf(assetId))
        }
    }

    private suspend fun syncSwapSupportChains() {
        val nativeAssetIds = Chain.entries.map { it.asset().id.toIdentifier() }
        syncAvailability(
            currentEnabledAssetIds = assetsDao.getSwapAvailableAssetIds(nativeAssetIds),
            targetEnabledAssetIds = Chain.swapSupport()
                .map { it.asset().id.toIdentifier() },
            trackedAssetIds = nativeAssetIds,
            setAvailability = assetsDao::setSwapAvailable,
        )
    }

    private suspend fun syncAvailability(
        currentEnabledAssetIds: List<String>,
        targetEnabledAssetIds: List<String>,
        setAvailability: suspend (List<String>, Boolean) -> Unit,
        trackedAssetIds: List<String> = (currentEnabledAssetIds + targetEnabledAssetIds).distinct(),
    ) {
        val changes = calculateAvailabilityChanges(
            currentEnabledAssetIds = currentEnabledAssetIds,
            targetEnabledAssetIds = targetEnabledAssetIds,
            trackedAssetIds = trackedAssetIds,
        )

        if (changes.idsToDisable.isNotEmpty()) {
            setAvailability(changes.idsToDisable, false)
        }
        if (changes.idsToEnable.isNotEmpty()) {
            setAvailability(changes.idsToEnable, true)
        }
    }

    private suspend fun List<AssetInfo>.updateBalances(): List<Deferred<List<AssetBalance>>> = withContext(Dispatchers.IO) {
        groupBy { it.walletId }
            .mapValues { wallet ->
                val walletId = wallet.key ?: return@mapValues null
                wallet.value.groupBy { it.asset.chain }
                    .mapKeys { it.value.firstOrNull()?.owner }
                    .mapValues { entry -> entry.value.map { it.asset } }
                    .mapNotNull { entry ->
                        val account: Account = entry.key ?: return@mapNotNull null
                        if (entry.value.isEmpty()) {
                            return@mapNotNull null
                        }
                        async {
                            updateBalances.updateBalances(walletId, account, entry.value)
                        }
                    }
            }
            .mapNotNull { it.value }
            .flatten()
    }

    private suspend fun changeCurrency(currency: Currency) {
        val rate = pricesDao.getRates(currency).map { it?.toDTO() }.firstOrNull() ?: return
        pricesDao.getAll().firstOrNull()?.map {
            it.copy(value = (it.usdValue ?: 0.0) * rate.rate, currency = currency.string)
        }?.let { pricesDao.insert(it) }
    }

    fun getCurrencyRate(currency: Currency): Flow<FiatRate?> {
        return pricesDao.getRates(currency).map { it?.toDTO() }
    }

    suspend fun addRecentActivity(
        assetId: AssetId,
        walletId: String,
        type: RecentType,
        toAssetId: AssetId? = null,
    ) {
        return assetsDao.addRecentActivity(
            DbRecentActivity(
                assetId = assetId.toIdentifier(),
                walletId = walletId,
                toAssetId = toAssetId?.toIdentifier(),
                type = type,
                addedAt = System.currentTimeMillis(),
            )
        )
    }

    fun getRecentAssets(request: RecentAssetsRequest): Flow<List<RecentAsset>> {
        return assetsDao.getRecentAssets(request.types, request.filters, request.limit)
            .map { items ->
                items.mapNotNull { row ->
                    val asset = row.asset.toDTO() ?: return@mapNotNull null
                    RecentAsset(asset = asset, addedAt = row.addedAt)
                }
            }
    }

    suspend fun clearRecentAssets(types: List<RecentType>) {
        assetsDao.clearRecentAssets(types)
    }
}

private fun DbAsset.toBasicUpdateRecord() = DbAssetBasicUpdate(
    id = id,
    name = name,
    symbol = symbol,
    decimals = decimals,
    type = type,
    chain = chain,
    isEnabled = isEnabled,
    isBuyEnabled = isBuyEnabled,
    isSellEnabled = isSellEnabled,
    isSwapEnabled = isSwapEnabled,
    isStakeEnabled = isStakeEnabled,
    stakingApr = stakingApr,
    rank = rank,
)

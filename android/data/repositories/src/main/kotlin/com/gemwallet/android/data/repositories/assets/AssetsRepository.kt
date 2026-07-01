package com.gemwallet.android.data.repositories.assets

import android.util.Log
import com.gemwallet.android.blockchain.operators.GetAsset
import com.gemwallet.android.blockchain.services.BalancesService
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.data.repositories.stream.StreamSubscriptionService
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.BalancesDao
import com.gemwallet.android.data.service.store.database.PricesDao
import com.gemwallet.android.data.service.store.database.entities.DbAsset
import com.gemwallet.android.data.service.store.database.entities.DbAssetBasicUpdate
import com.gemwallet.android.data.service.store.database.entities.DbBalance
import com.gemwallet.android.data.service.store.database.entities.toAssetInfoModel
import com.gemwallet.android.data.service.store.database.entities.toAssetLinkRecord
import com.gemwallet.android.data.service.store.database.entities.toAssetLinksModel
import com.gemwallet.android.data.service.store.database.entities.toMarketRecord
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toPriceRecord
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.data.service.store.database.entities.toUpdateRecord
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.domains.asset.defaultBasic
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.available
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.NO_QUERY_LIMIT
import com.gemwallet.android.model.RecentAsset
import com.gemwallet.android.model.RecentAssetsRequest
import com.gemwallet.android.model.RecentType
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetBasic
import com.wallet.core.primitives.AssetFull
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetLink
import com.wallet.core.primitives.AssetList
import com.wallet.core.primitives.AssetMarket
import com.wallet.core.primitives.AssetTag
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FiatRate
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

private const val TAG = "AssetsRepository"

@OptIn(ExperimentalCoroutinesApi::class)
@Singleton
class AssetsRepository @Inject constructor(
    private val assetsDao: AssetsDao,
    private val balancesDao: BalancesDao,
    private val pricesDao: PricesDao,
    private val sessionRepository: SessionRepository,
    private val balancesService: BalancesService,
    private val searchTokensCase: SearchTokensCase,
    private val streamSubscriptionService: StreamSubscriptionService,
    private val availabilityService: AssetsAvailabilityService,
    private val currencyRatesService: CurrencyRatesService,
    private val searchService: AssetsSearchService,
    private val recentAssetsService: RecentAssetsService,
    private val updateBalances: UpdateBalances = UpdateBalances(balancesDao, balancesService),
    private val scope: CoroutineScope = CoroutineScope(Dispatchers.IO),
) : GetAsset {


    init {
        scope.launch(Dispatchers.IO) {
            sessionRepository.session().collectLatest {
                currencyRatesService.changeCurrency(it?.currency ?: return@collectLatest)
            }
        }
        scope.launch(Dispatchers.IO) {
            availabilityService.syncSwapSupportChains()
        }
    }

    private fun currentWalletId(): Flow<String> = sessionRepository.currentWalletId()

    suspend fun sync() {
        getAssetsInfo().firstOrNull()?.refreshBalances()?.awaitAll()
    }

    suspend fun saveAssetMetadata(assetFull: AssetFull) = withContext(Dispatchers.IO) {
        val assetId = assetFull.asset.id
        val assetIdIdentifier = assetId.toIdentifier()
        val currency = sessionRepository.getCurrentCurrency()
        val rate = currencyRatesService.getCurrencyRate(currency).firstOrNull() ?: when (currency) {
            Currency.USD -> FiatRate(Currency.USD.string, 1.0)
            else -> null
        }
        val record = assetFull.toRecord().copy(
            updatedAt = System.currentTimeMillis(),
        )
        val linkRecords = assetFull.links.toAssetLinkRecord(assetId)
        val marketRecord = rate?.let { assetFull.toMarketRecord(it.rate) }
        assetsDao.upsertAssetMetadata(record, linkRecords, marketRecord)
        rate?.let { fiatRate ->
            val currentPrice = pricesDao.getByAssets(listOf(assetIdIdentifier)).firstOrNull()
            val priceRecord = assetFull.toPriceRecord(fiatRate)
            if (priceRecord != null && (currentPrice == null || currentPrice.currency != priceRecord.currency || currentPrice.value == null)) {
                pricesDao.insert(priceRecord)
            }
        }
    }

    suspend fun updateAssetMarket(assetId: AssetId, market: AssetMarket, currency: Currency) = withContext(Dispatchers.IO) {
        val rate = currencyRatesService.getCurrencyRate(currency).firstOrNull()?.rate ?: return@withContext
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
            insertLocalAsset(wallet.id.id, asset, isVisible)
            if (isVisible) assetIds.add(asset.id)
        }
        if (assetIds.isNotEmpty()) {
            streamSubscriptionService.addAssetIds(assetIds)
        }
    }

    suspend fun getNativeAssets(wallet: Wallet): List<Asset> = withContext(Dispatchers.IO) {
        assetsDao.getNativeWalletAssets(wallet.id.id)
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

    private val assetsInfo: Flow<List<AssetInfo>> = currentWalletId()
        .flatMapLatest { walletId -> assetsDao.getAssetsInfo(walletId) }
        .toAssetInfoModel()
        .shareIn(scope, SharingStarted.Eagerly, replay = 1)

    fun getAssetsInfo(): Flow<List<AssetInfo>> = assetsInfo

    fun getAssetsInfo(assetsId: List<AssetId>): Flow<List<AssetInfo>> = currentWalletId()
        .flatMapLatest { walletId -> assetsDao.getAssetsInfo(walletId, assetsId.map { it.toIdentifier() }) }
        .toAssetInfoModel()
        .flowOn(Dispatchers.IO)


    fun getAssetInfo(assetId: AssetId): Flow<AssetInfo?> {
        return currentWalletId()
            .flatMapLatest { walletId -> assetsDao.getAssetInfo(walletId, assetId.toIdentifier(), assetId.chain) }
            .map { it?.toDTO() }
            .flowOn(Dispatchers.IO)
    }

    fun asset(assetId: AssetId): Flow<Asset?> {
        return assetsDao.getAsset(assetId.toIdentifier())
            .map { it?.toDTO() }
            .flowOn(Dispatchers.IO)
    }

    fun getToken(assetId: AssetId): Flow<Asset?> = currentWalletId()
        .flatMapLatest { walletId -> assetsDao.getTokenInfo(walletId, assetId.toIdentifier(), assetId.chain) }
        .map { it?.toDTO()?.asset }
        .flowOn(Dispatchers.IO)

    fun getTokenInfo(assetId: AssetId): Flow<AssetInfo?> {
        return currentWalletId().flatMapLatest { walletId ->
            assetsDao.getAssetInfo(walletId, assetId.toIdentifier(), assetId.chain).flatMapLatest { assetInfo ->
                if (assetInfo == null) {
                    assetsDao.getTokenInfo(walletId, assetId.toIdentifier(), assetId.chain).map { it?.toDTO() }
                } else {
                    flow { emit(assetInfo.toDTO()) }
                }
            }
        }
        .flowOn(Dispatchers.IO)
    }

    fun getTokensInfo(assetsId: List<String>): Flow<List<AssetInfo>> {
        return currentWalletId()
            .flatMapLatest { walletId -> assetsDao.getAssetsInfoByAllWallets(walletId, assetsId) }
            .toAssetInfoModel()
    }

    suspend fun getWidgetTokens(currency: Currency): List<AssetInfo> = withContext(Dispatchers.IO) {
        val widgetAssetIds = listOf(AssetId(Chain.Bitcoin), AssetId(Chain.Ethereum), AssetId(Chain.Solana))

        runCatching { searchTokensCase.search(widgetAssetIds, currency) }
        getTokensInfo(widgetAssetIds.map { it.toIdentifier() }).firstOrNull() ?: emptyList()
    }

    suspend fun searchToken(assetId: AssetId, currency: Currency): Boolean {
        return searchTokensCase.search(assetId, currency)
    }

    fun search(query: String, tags: List<AssetTag>, byAllWallets: Boolean, limit: Int = NO_QUERY_LIMIT): Flow<List<AssetInfo>> =
        searchService.search(query, tags, byAllWallets, limit)

    fun searchLists(query: String): Flow<List<AssetList>> = searchService.searchLists(query)

    fun searchListAssets(listId: String, limit: Int = NO_QUERY_LIMIT): Flow<List<AssetInfo>> =
        searchService.searchListAssets(listId, limit)

    fun swapSearch(wallet: Wallet, query: String, byChains: List<Chain>, byAssets: List<AssetId>, tags: List<AssetTag>): Flow<List<AssetInfo>> =
        searchService.swapSearch(wallet, query, byChains, byAssets, tags)

    /**
     * Check and add new coins and active tokens
     * */
    fun invalidateDefault(wallet: Wallet) = scope.launch(Dispatchers.IO) {
        val assets = getNativeAssets(wallet).associateBy( { it.id.toIdentifier() }, { it })

        wallet.accounts.map { account ->
            val asset = account.chain.asset()
            async {
                if (assets[account.chain.string] == null) {
                    add(wallet.id.id, asset, false)
                    val balances = updateBalances.updateBalances(wallet.id.id, account, emptyList()).firstOrNull()
                    if ((balances?.totalAmount ?: 0.0) > 0.0) {
                        linkAssetToWallet(wallet.id.id, asset.id, true)
                    }
                }
            }
        }.awaitAll()
    }

    suspend fun switchVisibility(
        walletId: WalletId,
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
            linkAssetToWallet(walletId.id, assetId, true)
            updateBalances(assetId)
            return@withContext
        }
        if (isVisible == visibility) {
            return@withContext
        }
        linkAssetToWallet(walletId.id, assetId, visibility)
        if (visibility) {
            updateBalances(assetId)
        }
    }

    suspend fun togglePin(walletId: String, assetId: AssetId) = withContext(Dispatchers.IO) {
        assetsDao.toggleWalletAssetPin(walletId, assetId.toIdentifier())
    }

    suspend fun updateBalances(vararg tokens: AssetId) {
        getAssetsInfo(tokens.toList()).firstOrNull()?.refreshBalances()?.awaitAll()
    }

    suspend fun updateBalances(assetInfos: List<AssetInfo>) {
        assetInfos.refreshBalances().awaitAll()
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
        runCatching {
            assetsDao.insert(assets.map { it.toRecord() })
            assetsDao.updateBasicAssets(assets.map { it.toUpdateRecord() })
        }
            .onFailure { Log.e(TAG, "Failed to insert ${assets.size} assets", it) }
    }

    suspend fun linkAssetToWallet(
        walletId: String,
        assetId: AssetId,
        visible: Boolean,
    ) = withContext(Dispatchers.IO) {
        assetsDao.setWalletAssetVisibility(walletId, assetId.toIdentifier(), visible)
        if (visible) {
            streamSubscriptionService.addAssetIds(listOf(assetId))
        }
    }

    suspend fun addBalancesIfMissing(walletId: WalletId, assetIds: List<AssetId>) = withContext(Dispatchers.IO) {
        if (assetIds.isEmpty()) {
            return@withContext
        }
        val existing = assetsDao.getAssetIds(assetIds.map { it.toIdentifier() }).toSet()
        for (assetIdentifier in existing) {
            assetsDao.insertBalance(
                DbBalance(
                    assetId = assetIdentifier,
                    walletId = walletId.id,
                    isVisible = false,
                    updatedAt = null,
                )
            )
        }
    }

    suspend fun updateNativeAssetRanks() = withContext(Dispatchers.IO) {
        val basics = Chain.available().map { it.asset().defaultBasic }
        assetsDao.insert(basics.map { it.toRecord() })
        for (basic in basics) {
            runCatching { assetsDao.updateAssetRank(basic.asset.id.toIdentifier(), basic.score.rank) }
                .onFailure { Log.e(TAG, "Failed to update native asset rank for ${basic.asset.id}", it) }
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
        assetsDao.updateBasicAssets(listOf(record.toBasicUpdateRecord()))
        assetsDao.setWalletAssetVisibility(walletId, assetIdIdentifier, visible)
    }

    suspend fun updateBuyAvailable(assets: List<String>) = availabilityService.updateBuyAvailable(assets)

    suspend fun updateSellAvailable(assets: List<String>) = availabilityService.updateSellAvailable(assets)

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

    private suspend fun List<AssetInfo>.refreshBalances(): List<Deferred<List<AssetBalance>>> = withContext(Dispatchers.IO) {
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
                            updateBalances.updateBalances(walletId.id, account, entry.value)
                        }
                    }
            }
            .mapNotNull { it.value }
            .flatten()
    }

    fun getCurrencyRate(currency: Currency): Flow<FiatRate?> = currencyRatesService.getCurrencyRate(currency)

    suspend fun addRecentActivity(
        assetId: AssetId,
        walletId: String,
        type: RecentType,
        toAssetId: AssetId? = null,
    ) = recentAssetsService.addRecentActivity(assetId, walletId, type, toAssetId)

    fun getRecentAssets(request: RecentAssetsRequest): Flow<List<RecentAsset>> = recentAssetsService.getRecentAssets(request)

    suspend fun clearRecentAssets(walletId: WalletId, types: List<RecentType>) =
        recentAssetsService.clearRecentAssets(walletId, types)
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

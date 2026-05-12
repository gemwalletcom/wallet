package com.gemwallet.android.data.coordinators.wallet_import.services

import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
import com.gemwallet.android.application.wallet_import.coordinators.GetAvailableAssetIds
import com.gemwallet.android.application.wallet_import.coordinators.GetImportWalletState
import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletConfiguration
import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletImport
import com.gemwallet.android.application.wallet_import.values.ImportWalletState
import com.gemwallet.android.cases.device.SyncSubscription
import com.gemwallet.android.cases.nft.SyncNfts
import com.gemwallet.android.cases.tokens.SearchTokensCase
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.domains.asset.chain
import com.gemwallet.android.ext.getAccount
import com.gemwallet.android.ext.identifier
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.ext.type
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetSubtype
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineExceptionHandler
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.supervisorScope

class ImportWalletService(
    private val sessionRepository: SessionRepository,
    private val getAvailableAssetIds: GetAvailableAssetIds,
    private val searchTokensCase: SearchTokensCase,
    private val assetsRepository: AssetsRepository,
    private val syncSubscription: SyncSubscription,
    private val syncTransactions: SyncTransactions,
    private val syncNfts: SyncNfts,
    private val walletConfigurationSync: SyncWalletConfiguration,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO + CoroutineExceptionHandler { _, _ -> }),
) : SyncWalletImport, GetImportWalletState {

    private val importingWalletIds = MutableStateFlow<Set<WalletId>>(emptySet())

    override fun sync(wallet: Wallet) {
        importingWalletIds.update { it + wallet.id }
        scope.launch {
            try {
                syncWallet(wallet)
            } finally {
                importingWalletIds.update { it - wallet.id }
            }
        }
    }

    private suspend fun syncWallet(wallet: Wallet) {
        syncSubscription.syncSubscription(listOf(wallet))
        supervisorScope {
            launch { walletConfigurationSync.sync(wallet.id) }
            launch { discoverAssets(wallet) }
            launch { syncTransactions.syncTransactions(wallet) }
            launch { syncNfts.sync(wallet.id) }
        }
    }

    private suspend fun discoverAssets(wallet: Wallet) {
        val availableAssetsId = getAvailableAssetIds(wallet.id.id)
        val assetIds = availableAssetsId.mapNotNull { it.toAssetId() }
        val tokenIds = assetIds.filter { it.type() != AssetSubtype.NATIVE }

        searchTokensCase.search(tokenIds, sessionRepository.getCurrentCurrency())
        val assets = assetsRepository.getTokensInfo(assetIds.map { it.identifier }).firstOrNull().orEmpty()

        val linkedIds = assets.mapNotNull { assetInfo ->
            val asset = assetInfo.asset
            wallet.getAccount(asset.chain) ?: return@mapNotNull null
            assetsRepository.linkAssetToWallet(
                walletId = wallet.id.id,
                assetId = asset.id,
                visible = true,
            )
            asset.id
        }
        if (linkedIds.isNotEmpty()) {
            assetsRepository.updateBalances(*linkedIds.toTypedArray())
        }
    }

    override fun getImportState(walletId: WalletId): Flow<ImportWalletState> = importingWalletIds.map { walletIds ->
        if (walletId in walletIds) ImportWalletState.Importing else ImportWalletState.Complete
    }
}

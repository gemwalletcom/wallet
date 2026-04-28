// TODO: Out to special module or reorganize coordinators module as example rename to application or domain-application
package com.gemwallet.android.data.coordinators.wallet_import.services

import android.util.Log
import com.gemwallet.android.application.wallet_import.coordinators.GetAvailableAssetIds
import com.gemwallet.android.application.wallet_import.coordinators.GetImportWalletState
import com.gemwallet.android.application.wallet_import.services.ImportAssets
import com.gemwallet.android.application.transactions.coordinators.SyncTransactions
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
import com.wallet.core.primitives.AssetSubtype
import com.wallet.core.primitives.Wallet
import kotlinx.coroutines.CoroutineExceptionHandler
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.mapLatest
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

@OptIn(ExperimentalCoroutinesApi::class)
class ImportWalletService(
    private val sessionRepository: SessionRepository,
    private val getAvailableAssetIds: GetAvailableAssetIds,
    private val searchTokensCase: SearchTokensCase,
    private val assetsRepository: AssetsRepository,
    private val syncSubscription: SyncSubscription,
    private val syncTransactions: SyncTransactions,
    private val syncNfts: SyncNfts,
    private val scope: CoroutineScope = CoroutineScope(SupervisorJob() + Dispatchers.IO + CoroutineExceptionHandler {_, _ -> }),
) : ImportAssets, GetImportWalletState {

    private val jobs = MutableStateFlow<Map<String, Job>>(mutableMapOf())

    override fun importAssets(wallet: Wallet) {
        val job = scope.launch {
            syncSubscription.syncSubscription(listOf(wallet))
            val currency = sessionRepository.getCurrentCurrency()
            try {
                val availableAssetsId = getAvailableAssetIds(wallet.id)
                val assetIds = availableAssetsId.mapNotNull { it.toAssetId() }
                val tokenIds = assetIds.filter { it.type() != AssetSubtype.NATIVE }

                searchTokensCase.search(tokenIds, currency)
                val assets = assetsRepository.getTokensInfo(assetIds.map { it.identifier }).firstOrNull()
                val assetsToImport = assets?.map { it.asset }
                    ?.mapNotNull { asset ->
                        wallet.getAccount(asset.chain) ?: return@mapNotNull null
                        asset
                    } ?: emptyList()
                assetsRepository.add(
                    walletId = wallet.id,
                    assets = assetsToImport,
                    visible = true,
                )
                assetsRepository.sync()
                syncTransactions.syncTransactions(wallet)
                syncNfts.syncNfts(wallet)
            } catch (err: Throwable) {
                Log.d("IMPORT_ERROR", "Error:", err)
            } finally {
                jobs.update { entries -> entries.toMutableMap().apply { remove(wallet.id) } }
            }
        }
        jobs.update { it.toMutableMap().apply { put(wallet.id, job) } }
    }

    override fun getImportState(walletId: String): Flow<ImportWalletState> = jobs.mapLatest { entries ->
        when (entries[walletId]?.isActive) {
            true -> ImportWalletState.Importing
            else -> ImportWalletState.Complete
        }
    }
}

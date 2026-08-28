package com.gemwallet.android.data.coordinators.wallet_import.services

import com.gemwallet.android.application.wallet_import.coordinators.GetImportWalletState
import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletImport
import com.gemwallet.android.application.wallet_import.values.ImportWalletState
import com.gemwallet.android.cases.device.SyncDevice
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.CoroutineExceptionHandler
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemAssetDiscoveryService

class ImportWalletService(
    private val discoveryService: GemAssetDiscoveryService,
    private val sessionRepository: SessionRepository,
    private val syncDevice: SyncDevice,
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
        syncDevice.syncDevice()
        discoverAssets(wallet)
    }

    private suspend fun discoverAssets(wallet: Wallet) {
        discoveryService.discover(wallet.id.id)
    }

    override fun getImportState(walletId: WalletId): Flow<ImportWalletState> = importingWalletIds.map { walletIds ->
        if (walletId in walletIds) ImportWalletState.Importing else ImportWalletState.Complete
    }
}

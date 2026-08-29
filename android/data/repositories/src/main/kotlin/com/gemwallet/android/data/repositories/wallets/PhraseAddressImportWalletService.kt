package com.gemwallet.android.data.repositories.wallets

import com.gemwallet.android.application.wallet_import.cases.SyncWalletImport
import com.gemwallet.android.application.wallet_import.values.ImportError
import com.gemwallet.android.application.wallet_import.cases.ImportWalletService
import com.gemwallet.android.application.wallet_import.values.WalletImportResult
import com.gemwallet.android.application.wallet.cases.SetCurrentWallet
import com.gemwallet.android.ext.available
import com.gemwallet.android.ext.words
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import android.util.Log
import com.gemwallet.android.serializer.toJson
import uniffi.gemstone.GemAppStartService
import uniffi.gemstone.GemWalletImportException
import uniffi.gemstone.GemWalletImportResult
import uniffi.gemstone.GemWalletImportType
import uniffi.gemstone.GemWalletService
import uniffi.gemstone.GemWalletSource
import uniffi.gemstone.GemDeviceService

class PhraseAddressImportWalletService(
    private val walletService: GemWalletService,
    private val setCurrentWallet: SetCurrentWallet,
    private val appStartService: GemAppStartService,
    private val deviceService: GemDeviceService,
    private val walletImportSync: SyncWalletImport,
) : ImportWalletService {

    override suspend fun importWallet(
        importType: ImportType,
        walletName: String,
        data: String
    ): WalletImportResult {
        val import = when (importType.walletType) {
            WalletType.Multicoin -> GemWalletImportType.MulticoinPhrase(
                words = data.words(),
                chains = Chain.entries.filter(Chain.available()::contains).map { it.string },
            )
            WalletType.Single -> GemWalletImportType.SinglePhrase(words = data.words(), chain = importedChain(importType).string)
            WalletType.View -> GemWalletImportType.Address(address = data, chain = importedChain(importType).string)
            WalletType.PrivateKey -> GemWalletImportType.PrivateKey(value = data, chain = importedChain(importType).string)
        }
        return when (val result = import(walletName, validated(import), GemWalletSource.IMPORT)) {
            is GemWalletImportResult.Existing -> WalletImportResult.Existing(result.wallet.decodeJson())
            is GemWalletImportResult.New -> {
                val wallet = result.wallet.decodeJson<Wallet>()
                setupWallet(wallet)
                walletImportSync.sync(wallet)
                WalletImportResult.New(wallet)
            }
        }
    }

    override suspend fun createWallet(walletName: String, data: String): Wallet {
        val import = GemWalletImportType.MulticoinPhrase(
            words = data.words(),
            chains = Chain.entries.filter(Chain.available()::contains).map { it.string },
        )
        val wallet = when (val result = import(walletName, validated(import), GemWalletSource.CREATE)) {
            is GemWalletImportResult.Existing -> result.wallet.decodeJson<Wallet>()
            is GemWalletImportResult.New -> result.wallet.decodeJson<Wallet>()
        }
        setupWallet(wallet)
        deviceService.synchronizeIfNeeded()
        return wallet
    }

    private fun importedChain(importType: ImportType): Chain =
        requireNotNull(importType.chain) { "${importType.walletType} import requires a chain" }

    private suspend fun import(walletName: String, import: GemWalletImportType, source: GemWalletSource): GemWalletImportResult {
        return try {
            walletService.importWallet(walletName, import, source)
        } catch (error: Exception) {
            throw ImportError.CreateError(error.message.orEmpty())
        }
    }

    private suspend fun setupWallet(wallet: Wallet) {
        appStartService.setupWallet(wallet.toJson()).forEach { failure ->
            Log.e(TAG, "${failure.step} failed for ${wallet.id.id}: ${failure.message}")
        }
        setCurrentWallet.setCurrentWallet(wallet.id)
    }

    private fun validated(import: GemWalletImportType): GemWalletImportType {
        return try {
            walletService.validateImport(import)
        } catch (error: GemWalletImportException) {
            throw when (error) {
                is GemWalletImportException.InvalidSecretPhraseWords -> ImportError.InvalidWords(error.words)
                is GemWalletImportException.InvalidSecretPhrase -> ImportError.InvalidationSecretPhrase
                is GemWalletImportException.InvalidPrivateKey -> ImportError.InvalidationPrivateKey
                is GemWalletImportException.InvalidAddress -> ImportError.InvalidAddress
            }
        }
    }

    companion object {
        private const val TAG = "ImportWallet"

        fun decodePrivateKey(chain: Chain, data: String): ByteArray {
            return uniffi.gemstone.decodePrivateKey(chain = chain.string, value = data)
        }
    }
}

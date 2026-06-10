package com.gemwallet.android.data.repositories.wallets

import com.gemwallet.android.application.PasswordStore
import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletImport
import com.gemwallet.android.blockchain.operators.InvalidPhrase
import com.gemwallet.android.blockchain.operators.InvalidWords
import com.gemwallet.android.blockchain.operators.StorePhraseOperator
import com.gemwallet.android.blockchain.operators.ValidateAddressOperator
import com.gemwallet.android.blockchain.operators.ValidatePhraseOperator
import com.gemwallet.android.cases.device.SyncSubscription
import com.gemwallet.android.cases.wallet.ImportError
import com.gemwallet.android.cases.wallet.ImportWalletService
import com.gemwallet.android.cases.wallet.WalletImportResult
import com.gemwallet.android.data.repositories.assets.AssetsRepository
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.words
import com.gemwallet.android.math.hex
import com.gemwallet.android.model.ImportType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType

class PhraseAddressImportWalletService(
    private val walletsRepository: WalletsRepository,
    private val assetsRepository: AssetsRepository,
    private val sessionRepository: SessionRepository,
    private val storePhraseOperator: StorePhraseOperator,
    private val phraseValidate: ValidatePhraseOperator,
    private val addressValidate: ValidateAddressOperator,
    private val passwordStore: PasswordStore,
    private val syncSubscription: SyncSubscription,
    private val walletImportSync: SyncWalletImport,
) : ImportWalletService {

    override suspend fun importWallet(
        importType: ImportType,
        walletName: String,
        data: String
    ): WalletImportResult {
        val wallet = try {
            when (importType.walletType) {
                WalletType.Multicoin -> handlePhrase(importType, walletName, data, WalletSource.Import)
                WalletType.Single -> handlePhrase(importType, walletName, data, WalletSource.Import)
                WalletType.View -> handleAddress(importType.chain!!, walletName, data)
                WalletType.PrivateKey -> handlePrivateKey(importType.chain!!, walletName, data)
            }
        } catch (err: ImportError.DuplicatedWallet) {
            return WalletImportResult.Existing(err.wallet)
        } // Other exception handle on call

        setupWallet(wallet)
        walletImportSync.sync(wallet)
        return WalletImportResult.New(wallet)
    }

    override suspend fun createWallet(walletName: String, data: String): Wallet {
        val wallet = handlePhrase(ImportType(WalletType.Multicoin), walletName, data, WalletSource.Create)

        setupWallet(wallet)
        syncSubscription.syncSubscription(listOf(wallet))

        return wallet
    }

    private suspend fun setupWallet(wallet: Wallet) {
        assetsRepository.createAssets(wallet)
        sessionRepository.setWallet(wallet)
    }

    private suspend fun handlePhrase(importType: ImportType, walletName: String, rawData: String, source: WalletSource): Wallet {
        val cleanedData = rawData.words().joinToString(" ")
        val validateResult = phraseValidate(cleanedData)
        if (validateResult.isFailure || validateResult.getOrNull() != true) {
            val error = validateResult.exceptionOrNull() ?: InvalidPhrase
            throw when (error) {
                is InvalidWords -> ImportError.InvalidWords(error.words)
                else -> ImportError.InvalidationSecretPhrase
            }
        }
        val wallet = walletsRepository.addControlled(walletName, cleanedData, importType.walletType, importType.chain, source)

        val password = passwordStore.createPassword(wallet.id.id)
        val storeResult = storePhraseOperator(wallet, cleanedData, password)
        return if (storeResult.isSuccess) {
            wallet
        } else {
            walletsRepository.removeWallet(wallet.id)
            throw storeResult.exceptionOrNull() ?: ImportError.CreateError("Unknown error")
        }
    }

    private suspend fun handleAddress(chain: Chain, walletName: String, data: String): Wallet {
        if (addressValidate(data, chain).getOrNull() != true) {
            throw ImportError.InvalidAddress
        }
        return try {
            walletsRepository.addWatch(walletName, data, chain)
        } catch (err: ImportError.DuplicatedWallet) {
            throw err
        } catch (err: Exception) {
            throw ImportError.CreateError(err.message ?: "Unknown error")
        }
    }

    private suspend fun handlePrivateKey(chain: Chain, walletName: String, data: String): Wallet {
        val key = try {
            val data = decodePrivateKey(chain, data.trim())
            data.hex
        } catch (_: Throwable) {
            throw ImportError.InvalidationPrivateKey
        }

        val wallet = walletsRepository.addControlled(walletName, key, WalletType.PrivateKey, chain, source = WalletSource.Import)

        val password = passwordStore.createPassword(wallet.id.id)
        val storeResult = storePhraseOperator(wallet, key, password)
        return if (storeResult.isSuccess) {
            wallet
        } else {
            walletsRepository.removeWallet(wallet.id)
            throw storeResult.exceptionOrNull() ?: ImportError.CreateError("Unknown error")
        }
    }

    companion object {
        fun decodePrivateKey(chain: Chain, data: String): ByteArray {
            return uniffi.gemstone.decodePrivateKey(chain = chain.string, value = data)
        }
    }

}

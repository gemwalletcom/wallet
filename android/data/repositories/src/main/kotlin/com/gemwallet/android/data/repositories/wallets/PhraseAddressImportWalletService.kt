package com.gemwallet.android.data.repositories.wallets

import com.gemwallet.android.application.wallet_import.coordinators.SyncWalletImport
import com.gemwallet.android.blockchain.operators.InvalidPhrase
import com.gemwallet.android.blockchain.operators.InvalidWords
import com.gemwallet.android.blockchain.operators.ValidatePhraseOperator
import com.gemwallet.android.cases.device.SyncDevice
import com.gemwallet.android.cases.wallet.ImportError
import com.gemwallet.android.cases.wallet.ImportWalletService
import com.gemwallet.android.cases.wallet.WalletImportResult
import com.gemwallet.android.data.repositories.session.SessionRepository
import com.gemwallet.android.ext.available
import com.gemwallet.android.ext.isValidAddress
import com.gemwallet.android.ext.words
import com.gemwallet.android.math.hex
import com.gemwallet.android.model.ImportType
import com.gemwallet.android.serializer.decodeJson
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemWalletImportResult
import uniffi.gemstone.GemWalletImportType
import uniffi.gemstone.GemWalletService
import uniffi.gemstone.GemWalletSource

class PhraseAddressImportWalletService(
    private val walletService: GemWalletService,
    private val sessionRepository: SessionRepository,
    private val phraseValidate: ValidatePhraseOperator,
    private val syncDevice: SyncDevice,
    private val walletImportSync: SyncWalletImport,
) : ImportWalletService {

    override suspend fun importWallet(
        importType: ImportType,
        walletName: String,
        data: String
    ): WalletImportResult {
        val chain = importType.chain
        val import = when (importType.walletType) {
            WalletType.Multicoin -> GemWalletImportType.MulticoinPhrase(
                words = validatedWords(data),
                chains = Chain.entries.filter(Chain.available()::contains).map { it.string },
            )
            WalletType.Single -> GemWalletImportType.SinglePhrase(words = validatedWords(data), chain = chain!!.string)
            WalletType.View -> GemWalletImportType.Address(address = validatedAddress(chain!!, data), chain = chain.string)
            WalletType.PrivateKey -> GemWalletImportType.PrivateKey(value = decodedPrivateKey(chain!!, data), chain = chain.string)
        }
        return when (val result = import(walletName, import, GemWalletSource.IMPORT)) {
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
            words = validatedWords(data),
            chains = Chain.entries.filter(Chain.available()::contains).map { it.string },
        )
        val wallet = when (val result = import(walletName, import, GemWalletSource.CREATE)) {
            is GemWalletImportResult.Existing -> result.wallet.decodeJson<Wallet>()
            is GemWalletImportResult.New -> result.wallet.decodeJson<Wallet>()
        }
        setupWallet(wallet)
        syncDevice.syncDevice()
        return wallet
    }

    private suspend fun import(walletName: String, import: GemWalletImportType, source: GemWalletSource): GemWalletImportResult {
        return try {
            walletService.importWallet(walletName, import, source)
        } catch (error: Exception) {
            throw ImportError.CreateError(error.message ?: "Unknown error")
        }
    }

    private suspend fun setupWallet(wallet: Wallet) {
        sessionRepository.setWallet(wallet)
    }

    private fun validatedWords(rawData: String): List<String> {
        val words = rawData.words()
        val validateResult = phraseValidate(words.joinToString(" "))
        if (validateResult.isFailure || validateResult.getOrNull() != true) {
            val error = validateResult.exceptionOrNull() ?: InvalidPhrase
            throw when (error) {
                is InvalidWords -> ImportError.InvalidWords(error.words)
                else -> ImportError.InvalidationSecretPhrase
            }
        }
        return words
    }

    private fun validatedAddress(chain: Chain, data: String): String {
        if (!chain.isValidAddress(data)) {
            throw ImportError.InvalidAddress
        }
        return data
    }

    private fun decodedPrivateKey(chain: Chain, data: String): String {
        return try {
            decodePrivateKey(chain, data.trim()).hex
        } catch (_: Throwable) {
            throw ImportError.InvalidationPrivateKey
        }
    }

    companion object {
        fun decodePrivateKey(chain: Chain, data: String): ByteArray {
            return uniffi.gemstone.decodePrivateKey(chain = chain.string, value = data)
        }
    }
}

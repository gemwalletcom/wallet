package com.gemwallet.android.data.repositories.wallets

import com.gemwallet.android.application.wallet.coordinators.WalletIdGenerator
import com.gemwallet.android.blockchain.operators.CreateAccountOperator
import com.gemwallet.android.cases.wallet.ImportError
import com.gemwallet.android.data.repositories.addresses.AddressesRepository
import com.gemwallet.android.data.service.store.database.AccountsDao
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.WalletsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.domains.asset.defaultBasic
import com.gemwallet.android.ext.asset
import com.gemwallet.android.ext.available
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import com.wallet.core.primitives.WalletSource
import com.wallet.core.primitives.WalletType
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class WalletsRepositoryImpl @Inject constructor(
    private val walletsDao: WalletsDao,
    private val accountsDao: AccountsDao,
    private val addressesRepository: AddressesRepository,
    private val assetsDao: AssetsDao,
    private val createAccount: CreateAccountOperator,
    private val walletIdGenerator: WalletIdGenerator,
    private val transactionRunner: StoreTransactionRunner,
) : WalletsRepository {

    override suspend fun getNextWalletNumber(): Int {
        return getAll().map { it.size + 1 }.firstOrNull() ?: 0
    }

    override fun getAll(): Flow<List<Wallet>> = walletsDao.getAll().toDTO()

    override suspend fun addWatch(walletName: String, address: String, chain: Chain): Wallet {
        val walletId = walletIdGenerator.generateWalletId(WalletType.View, chain, address)
        val hasWallet = getWallet(walletId).firstOrNull()
        if (hasWallet != null) {
            throw ImportError.DuplicatedWallet(hasWallet)
        }

        return putWallet(
            Wallet(
                id = walletId,
                name = walletName,
                type = WalletType.View,
                accounts = listOf(
                    Account(
                        address = address,
                        chain = chain,
                        derivationPath = "",
                    )
                ),
                index = getNextWalletNumber(),
                isPinned = false,
                source = WalletSource.Import,
            )
        )
    }

    override suspend fun addControlled(
        walletName: String,
        data: String,
        type: WalletType,
        chain: Chain?,
        source: WalletSource
    ): Wallet {
        val accounts = mutableListOf<Account>()
        val availableChains = Chain.available()
        val chains =
            if ((type == WalletType.Single || type == WalletType.PrivateKey) && chain != null) listOf(
                chain
            ) else Chain.entries.filter(availableChains::contains)
        for (item in chains) {
            accounts.add(createAccount(type, data, item))
        }
        val priorityAccount = walletIdGenerator.getPriorityAccount(accounts)
        val walletId = walletIdGenerator.generateWalletId(type, priorityAccount!!.chain, priorityAccount.address)

        val hasWallet = getWallet(walletId).firstOrNull()
        if (hasWallet != null) {
            throw ImportError.DuplicatedWallet(hasWallet)
        }

        val wallet = Wallet(
            id = walletId,
            name = walletName,
            type = type,
            accounts = accounts,
            index = getNextWalletNumber(),
            isPinned = false,
            source = source,
        )
        return putWallet(wallet)
    }

    override suspend fun updateWallet(wallet: Wallet) {
        walletsDao.update(wallet.toRecord())
    }

    override suspend fun updateAccounts(wallet: Wallet) = withContext(Dispatchers.IO) {
        transactionRunner.run {
            insertAccountsWithNativeAssets(wallet)
        }
    }

    private suspend fun insertAccountsWithNativeAssets(wallet: Wallet) {
        insertNativeAssets(wallet.accounts)
        accountsDao.insert(wallet.accounts.map { it.toRecord(wallet.id.id) })
    }

    override suspend fun removeWallet(walletId: WalletId) = withContext(Dispatchers.IO) {
        val wallet = walletsDao.getById(walletId.id).firstOrNull() ?: return@withContext false
        accountsDao.deleteByWalletId(walletId.id)
        walletsDao.delete(wallet)
        true
    }

    override fun getWallet(walletId: WalletId): Flow<Wallet?> {
        return walletsDao.getById(walletId.id).map { walletRecord ->
            val accounts = accountsDao.getByWalletId(walletId.id)
            if (accounts.isEmpty()) return@map null
            walletRecord?.toDTO(accounts)
        }.flowOn(Dispatchers.IO)
    }

    private suspend fun putWallet(wallet: Wallet): Wallet = withContext(Dispatchers.IO) {
        transactionRunner.run {
            walletsDao.insert(wallet.toRecord())
            insertAccountsWithNativeAssets(wallet)
            addressesRepository.saveWalletAddresses(wallet)
            wallet
        }
    }

    private suspend fun insertNativeAssets(accounts: List<Account>) {
        val records = accounts
            .map { it.chain.asset().defaultBasic.toRecord() }
            .distinctBy { it.id }
        if (records.isEmpty()) {
            return
        }
        assetsDao.insert(records)
    }
}

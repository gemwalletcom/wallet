package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.service.store.database.AccountsDao
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.StoreTransactionRunner
import com.gemwallet.android.data.service.store.database.WalletsDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.domains.asset.defaultBasic
import com.gemwallet.android.ext.asset
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.withContext
import uniffi.gemstone.GemWalletStore

class GemstoneWalletStore(
    private val walletsDao: WalletsDao,
    private val accountsDao: AccountsDao,
    private val assetsDao: AssetsDao,
    private val addressStore: GemstoneAddressStore,
    private val transactionRunner: StoreTransactionRunner,
) : GemWalletStore {

    override fun getWallets(): List<String> = getAllNow().map { it.toJson() }

    override fun getWallet(walletId: String): String? = getWalletNow(WalletId(walletId))?.toJson()

    override suspend fun addWallet(wallet: String) {
        addWallet(wallet.decodeJson<Wallet>())
    }

    override suspend fun deleteWallet(walletId: String): Boolean = removeWallet(WalletId(walletId))

    override suspend fun setPinned(walletId: String, pinned: Boolean) = setPinned(WalletId(walletId), pinned)

    override suspend fun setName(walletId: String, name: String) = rename(WalletId(walletId), name)

    override suspend fun setImageUrl(walletId: String, imageUrl: String?) = setImageUrl(WalletId(walletId), imageUrl)

    fun observeWallets(): Flow<List<Wallet>> = walletsDao.getAll().toDTO()

    fun observeWallet(walletId: WalletId): Flow<Wallet?> = walletsDao.getById(walletId.id)
        .map { record -> record?.toDTO(accountsDao.getByWalletId(walletId.id)) }
        .flowOn(Dispatchers.IO)

    fun getAllNow(): List<Wallet> = walletsDao.getAllNow().toDTO()

    fun getWalletNow(walletId: WalletId): Wallet? = walletsDao.getByIdNow(walletId.id).toDTO().firstOrNull()

    suspend fun addWallet(wallet: Wallet): Wallet = withContext(Dispatchers.IO) {
        transactionRunner.run {
            walletsDao.insert(wallet.toRecord())
            insertAccountsWithNativeAssets(wallet)
            addressStore.saveWalletAddresses(wallet)
            wallet
        }
    }

    suspend fun setPinned(walletId: WalletId, pinned: Boolean) = walletsDao.setPinned(walletId.id, pinned)

    suspend fun rename(walletId: WalletId, name: String) = walletsDao.setName(walletId.id, name)

    suspend fun setImageUrl(walletId: WalletId, imageUrl: String?) = walletsDao.setImageUrl(walletId.id, imageUrl)

    suspend fun updateAccounts(wallet: Wallet) = withContext(Dispatchers.IO) {
        transactionRunner.run {
            insertAccountsWithNativeAssets(wallet)
        }
    }

    suspend fun removeWallet(walletId: WalletId): Boolean = withContext(Dispatchers.IO) {
        val wallet = walletsDao.getById(walletId.id).firstOrNull() ?: return@withContext false
        accountsDao.deleteByWalletId(walletId.id)
        walletsDao.delete(wallet)
        true
    }

    private suspend fun insertAccountsWithNativeAssets(wallet: Wallet) {
        insertNativeAssets(wallet.accounts)
        accountsDao.insert(wallet.accounts.map { it.toRecord(wallet.id.id) })
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

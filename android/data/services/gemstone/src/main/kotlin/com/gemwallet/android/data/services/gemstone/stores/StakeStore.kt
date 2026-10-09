package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.data.services.store.database.StakeDao
import com.gemwallet.android.data.services.store.database.entities.toDTO
import com.gemwallet.android.data.services.store.database.entities.toRecord
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.first
import uniffi.gemstone.GemDelegationRecord
import uniffi.gemstone.GemStakeStore

class GemstoneStakeStore(private val stakeDao: StakeDao) : GemStakeStore {

    override suspend fun getValidators(assetId: String, providerType: uniffi.gemstone.StakeProviderType): List<uniffi.gemstone.DelegationValidator> =
        stakeDao.getValidators(AssetId(assetId), providerType.toPrimitives()).first().toDTO().map {
            it.toGem()
        }

    override suspend fun saveValidators(validators: List<uniffi.gemstone.DelegationValidator>) = stakeDao.upsertValidators(validators.map { it.toPrimitives() }.toRecord())

    override suspend fun deactivateValidators(assetId: String, validatorIds: List<String>) {
        if (validatorIds.isEmpty()) {
            return
        }
        stakeDao.deactivateValidators(AssetId(assetId), validatorIds)
    }

    override suspend fun getDelegationIds(walletId: String, assetId: String, providerType: uniffi.gemstone.StakeProviderType): List<String> = stakeDao.getDelegationIds(WalletId(walletId), AssetId(assetId), providerType.toPrimitives())

    override suspend fun updateDelegations(walletId: String, delegations: List<GemDelegationRecord>, deleteIds: List<String>) {
        val wallet = WalletId(walletId)
        stakeDao.updateAndDeleteDelegations(wallet, delegations.map { it.delegation.toPrimitives().toRecord(it.id, wallet) }, deleteIds)
    }
}

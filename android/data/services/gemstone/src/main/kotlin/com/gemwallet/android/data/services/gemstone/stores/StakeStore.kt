package com.gemwallet.android.data.services.gemstone.stores

import com.gemwallet.android.ext.toGem
import com.gemwallet.android.ext.toPrimitives
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.StakeDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.data.service.store.database.entities.toModel
import com.wallet.core.primitives.Delegation
import com.wallet.core.primitives.DelegationBase
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import uniffi.gemstone.GemStakeStore
import com.wallet.core.primitives.AssetId

class GemstoneStakeStore(
    private val stakeDao: StakeDao,
    private val assetsDao: AssetsDao,
) : GemStakeStore {

    override suspend fun getApr(assetId: String, providerType: uniffi.gemstone.StakeProviderType): Double? {
        val asset = assetsDao.getAsset(assetId).first() ?: return null
        return when (providerType.toPrimitives()) {
            StakeProviderType.Stake -> asset.stakingApr
            StakeProviderType.Earn -> asset.earnApr
        }
    }

    override suspend fun getValidators(assetId: String, providerType: uniffi.gemstone.StakeProviderType): List<uniffi.gemstone.DelegationValidator> {
        return stakeDao.getValidators(AssetId(assetId), providerType.toPrimitives()).first().toDTO().map { it.toGem() }
    }

    override suspend fun saveValidators(validators: List<uniffi.gemstone.DelegationValidator>) =
        stakeDao.upsertValidators(validators.map { it.toPrimitives() }.toRecord())

    override suspend fun deactivateValidators(assetId: String, validatorIds: List<String>) {
        if (validatorIds.isEmpty()) {
            return
        }
        stakeDao.deactivateValidators(AssetId(assetId), validatorIds)
    }

    override suspend fun getDelegationIds(walletId: String, assetId: String, providerType: uniffi.gemstone.StakeProviderType): List<String> {
        return stakeDao.getDelegationIds(WalletId(walletId), AssetId(assetId), providerType.toPrimitives())
    }

    override suspend fun updateDelegations(walletId: String, delegations: List<String>, deleteIds: List<String>) {
        val wallet = WalletId(walletId)
        stakeDao.updateAndDeleteDelegations(wallet, delegations.map { it.decodeJson<DelegationBase>() }.toRecord(wallet), deleteIds)
    }

    fun observeValidators(assetId: AssetId, providerType: StakeProviderType): Flow<List<DelegationValidator>> =
        stakeDao.getValidators(assetId, providerType).map { it.toDTO() }

    suspend fun getValidator(assetId: AssetId, validatorId: String): DelegationValidator? =
        stakeDao.getValidator(assetId, validatorId)?.toDTO()

    fun observeDelegations(walletId: WalletId, assetId: AssetId): Flow<List<Delegation>> =
        stakeDao.getDelegations(walletId, assetId).map { rows -> rows.mapNotNull { it.toModel() } }

    fun observeDelegation(walletId: WalletId, validatorId: String, delegationId: String): Flow<Delegation?> =
        stakeDao.getDelegation(walletId, validatorId, delegationId).map { it?.toModel() }
}

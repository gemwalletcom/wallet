package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.AddressesDao
import com.gemwallet.android.data.service.store.database.StakeDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.DelegationBase
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.first
import uniffi.gemstone.GemStakeStore
import com.wallet.core.primitives.AssetId

class GemstoneStakeStore(
    private val stakeDao: StakeDao,
    private val assetsDao: AssetsDao,
    private val addressesDao: AddressesDao,
) : GemStakeStore {

    override suspend fun getApr(assetId: String, providerType: String): Double? {
        val asset = assetsDao.getAsset(assetId).first() ?: return null
        return when (providerType.decodeJson<StakeProviderType>()) {
            StakeProviderType.Stake -> asset.stakingApr
            StakeProviderType.Earn -> asset.earnApr
        }
    }

    override suspend fun getValidators(assetId: String, providerType: String): List<String> {
        return stakeDao.getValidators(AssetId(assetId), providerType.decodeJson<StakeProviderType>()).first().toDTO().map { it.toJson() }
    }

    override suspend fun saveValidators(validators: List<String>) =
        stakeDao.upsertValidators(validators.map { it.decodeJson<DelegationValidator>() }.toRecord())

    override suspend fun deactivateValidators(assetId: String, validatorIds: List<String>) {
        if (validatorIds.isEmpty()) {
            return
        }
        stakeDao.deactivateValidators(AssetId(assetId), validatorIds)
    }

    override suspend fun getDelegationIds(walletId: String, assetId: String, providerType: String): List<String> {
        return stakeDao.getDelegationIds(WalletId(walletId), AssetId(assetId), providerType.decodeJson<StakeProviderType>())
    }

    override suspend fun updateDelegations(walletId: String, delegations: List<String>, deleteIds: List<String>) {
        val wallet = WalletId(walletId)
        stakeDao.updateAndDeleteDelegations(wallet, delegations.map { it.decodeJson<DelegationBase>() }.toRecord(wallet), deleteIds)
    }

    override suspend fun saveAddressNames(names: List<String>) {
        val records = names.map { it.decodeJson<AddressName>() }
        if (records.isNotEmpty()) {
            addressesDao.updateNames(records.toRecord())
        }
    }
}

package com.gemwallet.android.data.repositories.gemstone

import com.gemwallet.android.cases.addresses.SaveAddressNames
import com.gemwallet.android.data.service.store.database.AssetsDao
import com.gemwallet.android.data.service.store.database.StakeDao
import com.gemwallet.android.data.service.store.database.entities.toDTO
import com.gemwallet.android.data.service.store.database.entities.toRecord
import com.gemwallet.android.ext.toAssetId
import com.gemwallet.android.serializer.decodeJson
import com.gemwallet.android.serializer.toJson
import com.wallet.core.primitives.AddressName
import com.wallet.core.primitives.DelegationBase
import com.wallet.core.primitives.DelegationValidator
import com.wallet.core.primitives.StakeProviderType
import com.wallet.core.primitives.WalletId
import kotlinx.coroutines.flow.first
import uniffi.gemstone.GemStakeStore

class GemstoneStakeStore(
    private val stakeDao: StakeDao,
    private val assetsDao: AssetsDao,
    private val saveAddressNames: SaveAddressNames,
) : GemStakeStore {

    override suspend fun getApr(assetId: String, providerType: String): Double? {
        val asset = assetsDao.getAsset(assetId).first() ?: return null
        return when (providerType.decodeJson<StakeProviderType>()) {
            StakeProviderType.Stake -> asset.stakingApr
            StakeProviderType.Earn -> asset.earnApr
        }
    }

    override suspend fun getValidators(assetId: String, providerType: String): List<String> {
        val id = assetId.toAssetId() ?: return emptyList()
        return stakeDao.getValidators(id, providerType.decodeJson<StakeProviderType>()).first().toDTO().map { it.toJson() }
    }

    override suspend fun saveValidators(validators: List<String>) =
        stakeDao.upsertValidators(validators.map { it.decodeJson<DelegationValidator>() }.toRecord())

    override suspend fun getDelegationIds(walletId: String, assetId: String, providerType: String): List<String> {
        val id = assetId.toAssetId() ?: return emptyList()
        return stakeDao.getDelegationIds(WalletId(walletId), id, providerType.decodeJson<StakeProviderType>())
    }

    override suspend fun updateDelegations(walletId: String, delegations: List<String>, deleteIds: List<String>) {
        val wallet = WalletId(walletId)
        stakeDao.updateAndDeleteDelegations(wallet, delegations.map { it.decodeJson<DelegationBase>() }.toRecord(wallet), deleteIds)
    }

    override suspend fun saveAddressNames(names: List<String>) =
        saveAddressNames.saveAddressNames(names.map { it.decodeJson<AddressName>() })
}

package com.gemwallet.android.data.services.store.database.entities

import com.gemwallet.android.ext.toAssetId
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetAssociation
import com.wallet.core.primitives.AssetData
import com.wallet.core.primitives.AssetMetaData
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Balance
import com.wallet.core.primitives.BalanceMetadata
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.Price
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

data class DbAssetInfo(
    val id: String,
    val name: String,
    val symbol: String,
    val decimals: Int,
    val type: AssetType,
    val pinned: Boolean?,
    val visible: Boolean?,
    val isBuyEnabled: Boolean,
    val isSellEnabled: Boolean,
    val isSwapEnabled: Boolean,
    val isStakeEnabled: Boolean,
    val stakingApr: Double?,
    val isEarnEnabled: Boolean = false,
    val earnApr: Double? = null,
    val assetRank: Int,
    val isEnabled: Boolean,
    val associations: List<AssetAssociation>,
    // account
    val address: String?,
    val walletId: String?,
    val derivationPath: String?,
    val chain: Chain,
    val extendedPublicKey: String?,
    // price
    val priceValue: Double?,
    val priceDayChanges: Double?,
    val priceUpdatedAt: Long?,
    val priceCurrency: Currency?,
    // balance
    val balanceAvailable: String,
    val balanceAvailableAmount: Double?,
    val balanceFrozen: String,
    val balanceFrozenAmount: Double?,
    val balanceLocked: String,
    val balanceLockedAmount: Double?,
    val balanceStaked: String,
    val balanceStakedAmount: Double?,
    val balancePending: String,
    val balancePendingAmount: Double?,
    val balanceRewards: String,
    val balanceRewardsAmount: Double?,
    val balanceReserved: String,
    val balanceReservedAmount: Double?,
    val balanceWithdrawable: String,
    val balanceWithdrawableAmount: Double?,
    val balancePendingUnconfirmed: String = "0",
    val balancePendingUnconfirmedAmount: Double? = null,
    val balanceEarn: String = "0",
    val balanceEarnAmount: Double? = null,
    val balanceTotalAmount: Double?,
    val balanceFiatTotalAmount: Double?,
    val balanceMetadata: BalanceMetadata?,
    val assetIsActive: Boolean?,
)

fun Flow<List<DbAssetInfo>>.toAssetDataModel() = map { it.toAssetDataModels() }

fun List<DbAssetInfo>.toAssetDataModels() = mapNotNull { it.toDTO() }

fun DbAssetInfo.toDTO(): AssetData? {
    val entity = this
    val assetId = entity.id.toAssetId() ?: return null
    return AssetData(
        asset = Asset(
            id = assetId,
            name = entity.name,
            symbol = entity.symbol,
            decimals = entity.decimals,
            type = entity.type,
        ),
        balance = Balance(
            available = entity.balanceAvailable.toBigInteger(),
            frozen = entity.balanceFrozen.toBigInteger(),
            locked = entity.balanceLocked.toBigInteger(),
            staked = entity.balanceStaked.toBigInteger(),
            pending = entity.balancePending.toBigInteger(),
            pendingUnconfirmed = entity.balancePendingUnconfirmed.toBigInteger(),
            rewards = entity.balanceRewards.toBigInteger(),
            reserved = entity.balanceReserved.toBigInteger(),
            earn = entity.balanceEarn.toBigInteger(),
            withdrawable = entity.balanceWithdrawable.toBigInteger(),
            metadata = entity.balanceMetadata,
        ),
        account = Account(
            chain = entity.chain,
            address = entity.address.orEmpty(),
            derivationPath = entity.derivationPath.orEmpty(),
            extendedPublicKey = entity.extendedPublicKey,
        ),
        price = if (entity.priceValue != null && entity.priceValue > 0 && entity.priceCurrency != null) {
            Price(
                price = entity.priceValue,
                priceChangePercentage24h = entity.priceDayChanges ?: 0.0,
                updatedAt = entity.priceUpdatedAt ?: 0,
            )
        } else {
            null
        },
        priceAlerts = emptyList(),
        metadata = AssetMetaData(
            isEnabled = entity.isEnabled,
            isBalanceEnabled = entity.visible == true,
            isBuyEnabled = entity.isBuyEnabled,
            isSellEnabled = entity.isSellEnabled,
            isSwapEnabled = entity.isSwapEnabled,
            isStakeEnabled = entity.isStakeEnabled,
            isEarnEnabled = entity.isEarnEnabled,
            isPinned = entity.pinned == true,
            isActive = assetIsActive != false,
            stakingApr = entity.stakingApr,
            earnApr = entity.earnApr,
            rankScore = entity.assetRank,
        ),
        associations = entity.associations,
    )
}

data class DbAssetFiatValue(val amount: Double, val price: Double, val priceChangePercentage24h: Double)

package com.gemwallet.android.data.services.store.database.entities

import com.gemwallet.android.ext.toIdentifier
import com.gemwallet.android.testkit.mockAsset
import com.gemwallet.android.testkit.mockAssetId
import com.gemwallet.android.testkit.mockNftAssetId
import com.gemwallet.android.testkit.mockNftCollectionId
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetAssociation
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTCollectionId
import com.wallet.core.primitives.NFTType
import com.wallet.core.primitives.TransactionDirection
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.TransactionState
import com.wallet.core.primitives.TransactionType
import com.wallet.core.primitives.VerificationStatus
import com.wallet.core.primitives.WalletConnectionState
import com.wallet.core.primitives.WalletId

fun mockDbAssetInfo(
    chain: Chain = Chain.Bitcoin,
    visible: Boolean? = true,
    isStakeEnabled: Boolean = false,
    assetRank: Int = 1,
    isEnabled: Boolean = true,
    walletId: String? = "wallet-1",
    priceValue: Double? = null,
    priceUpdatedAt: Long? = null,
    priceCurrency: Currency? = null,
    balanceAvailable: String = "0",
    assetIsActive: Boolean? = true,
    associations: List<AssetAssociation> = emptyList(),
) = DbAssetInfo(
    id = chain.string,
    name = "Asset",
    symbol = "AST",
    decimals = 18,
    type = AssetType.NATIVE,
    pinned = false,
    visible = visible,
    isBuyEnabled = true,
    isSellEnabled = true,
    isSwapEnabled = true,
    isStakeEnabled = isStakeEnabled,
    stakingApr = null,
    assetRank = assetRank,
    isEnabled = isEnabled,
    associations = associations,
    address = "address",
    walletId = walletId,
    derivationPath = "",
    chain = chain,
    extendedPublicKey = null,
    priceValue = priceValue,
    priceDayChanges = null,
    priceUpdatedAt = priceUpdatedAt,
    priceCurrency = priceCurrency,
    balanceAvailable = balanceAvailable,
    balanceAvailableAmount = null,
    balanceFrozen = "0",
    balanceFrozenAmount = null,
    balanceLocked = "0",
    balanceLockedAmount = null,
    balanceStaked = "0",
    balanceStakedAmount = null,
    balancePending = "0",
    balancePendingAmount = null,
    balanceRewards = "0",
    balanceRewardsAmount = null,
    balanceReserved = "0",
    balanceReservedAmount = null,
    balanceWithdrawable = "0",
    balanceWithdrawableAmount = null,
    balanceTotalAmount = null,
    balanceFiatTotalAmount = null,
    balanceMetadata = null,
    assetIsActive = assetIsActive,
)

fun mockDbConnection(id: String = "connection-1", walletId: String = "wallet-1") = DbConnection(
    id = id,
    walletId = walletId,
    sessionId = id,
    state = WalletConnectionState.Active,
    chains = listOf(Chain.Ethereum),
    createdAt = 1_000,
    expireAt = 2_000,
    appName = "App",
    appDescription = "Description",
    appUrl = "https://example.com",
    appIcon = "https://example.com/icon.png",
    redirectNative = null,
    redirectUniversal = null,
)

fun mockDbNftAsset(id: NFTAssetId = mockNftAssetId(), collectionId: NFTCollectionId = mockNftCollectionId()) = DbNFTAsset(
    id = id,
    collectionId = collectionId,
    tokenId = id.tokenId,
    tokenType = NFTType.ERC721,
    name = id.toIdentifier(),
    chain = id.chain,
    contractAddress = id.contractAddress,
    imageUrl = "",
)

fun mockDbNftCollection(id: NFTCollectionId = mockNftCollectionId()) = DbNFTCollection(
    id = id,
    name = id.toIdentifier(),
    chain = id.chain,
    contractAddress = id.contractAddress,
    imageUrl = "",
    status = VerificationStatus.Verified,
)

fun mockDbTransactionExtended(type: TransactionType = TransactionType.Transfer, priceValue: Double? = null, assets: List<Asset> = emptyList(), prices: List<DbPrice> = emptyList()): DbTransactionExtended {
    val asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18u)
    val id = TransactionId(asset.id.chain, "0xhash")
    return DbTransactionExtended(
        transaction = DbTransaction(
            id = id,
            walletId = WalletId("wallet-1"),
            hash = "0xhash",
            assetId = asset.id,
            feeAssetId = AssetId(chain = asset.id.chain),
            owner = "owner",
            recipient = "recipient",
            state = TransactionState.Confirmed,
            type = type,
            blockNumber = "1",
            sequence = "1",
            fee = "1",
            value = "5",
            direction = TransactionDirection.Outgoing,
            createdAt = 0,
            updatedAt = 0,
        ),
        asset = asset.toDbAssetProjection(),
        feeAsset = asset.toDbAssetProjection(),
        priceValue = priceValue,
        priceDayChanged = null,
        feePriceValue = null,
        feePriceDayChanged = null,
        fromAddress = null,
        toAddress = null,
        transactionKey = id.identifier,
        assets = assets.map { it.toDbAssetProjection() },
        prices = prices,
    )
}

private fun Asset.toDbAssetProjection() = DbAssetProjection(
    id = id.toIdentifier(),
    name = name,
    symbol = symbol,
    decimals = decimals.toInt(),
    type = type,
)

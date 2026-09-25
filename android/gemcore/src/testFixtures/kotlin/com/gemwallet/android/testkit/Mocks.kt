package com.gemwallet.android.testkit

import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.Balance
import com.gemwallet.android.model.ChainAssetInfo
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetAssociation
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetMetaData
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.BalanceMetadata
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTCollectionId
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.WalletId
import java.math.BigInteger

fun mockAssetId(chain: Chain = Chain.Bitcoin, tokenId: String? = null) = AssetId(
    chain = chain,
    tokenId = tokenId,
)

fun mockWalletId(address: String = "0x0000000000000000000000000000000000000000") = WalletId("multicoin_$address")

fun mockNftAssetId(chain: Chain = Chain.Bitcoin, contractAddress: String = "0xcontract", tokenId: String = "1") = NFTAssetId(
    chain = chain,
    contractAddress = contractAddress,
    tokenId = tokenId,
)

fun mockNftCollectionId(chain: Chain = Chain.Bitcoin, contractAddress: String = "0xcontract") = NFTCollectionId(
    chain = chain,
    contractAddress = contractAddress,
)

fun mockPerpetualId(provider: PerpetualProvider = PerpetualProvider.Hypercore, symbol: String = "BTC") = PerpetualId(
    provider = provider,
    symbol = symbol,
)

fun mockTransactionId(chain: Chain = Chain.Bitcoin, hash: String = "tx-id") = TransactionId(
    chain = chain,
    hash = hash,
)

fun mockAssetInfo(
    owner: Account? = null,
    asset: Asset = mockAsset(),
    balance: AssetBalance = mockAssetBalance(),
    walletId: WalletId? = null,
    price: AssetPriceInfo? = null,
    metadata: AssetMetaData = mockAssetMetaData(),
    associations: List<AssetAssociation> = emptyList(),
) = AssetInfo(
    owner = owner,
    asset = asset,
    balance = balance,
    walletId = walletId,
    price = price,
    metadata = metadata,
    associations = associations,
)

fun mockAssetBalance(asset: Asset = mockAsset(), balance: Balance = mockBalance(), metadata: BalanceMetadata? = null, isActive: Boolean = false) = AssetBalance(
    asset = asset,
    balance = balance,
    metadata = metadata,
    isActive = isActive,
)

fun mockBalance(
    available: BigInteger = BigInteger.ZERO,
    frozen: BigInteger = BigInteger.ZERO,
    locked: BigInteger = BigInteger.ZERO,
    staked: BigInteger = BigInteger.ZERO,
    pending: BigInteger = BigInteger.ZERO,
    rewards: BigInteger = BigInteger.ZERO,
    reserved: BigInteger = BigInteger.ZERO,
    withdrawable: BigInteger = BigInteger.ZERO,
    pendingUnconfirmed: BigInteger = BigInteger.ZERO,
    earn: BigInteger = BigInteger.ZERO,
) = Balance(
    available = available,
    frozen = frozen,
    locked = locked,
    staked = staked,
    pending = pending,
    rewards = rewards,
    reserved = reserved,
    withdrawable = withdrawable,
    pendingUnconfirmed = pendingUnconfirmed,
    earn = earn,
)

fun mockAssetPriceInfo(currency: Currency = Currency.MXN, price: AssetPrice = mockAssetPrice()) = AssetPriceInfo(
    currency = currency,
    price = price,
)

fun mockAssetPriceValue(asset: Asset = mockAsset(), price: AssetPriceInfo? = null) = AssetPriceValue(
    asset = asset,
    price = price,
)

fun mockChainAssetInfo(assetInfo: AssetInfo = mockAssetInfo(), feeAssetInfo: AssetInfo = mockAssetInfo()) = ChainAssetInfo(
    assetInfo = assetInfo,
    feeAssetInfo = feeAssetInfo,
)

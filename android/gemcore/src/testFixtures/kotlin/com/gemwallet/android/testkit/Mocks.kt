package com.gemwallet.android.testkit

import com.gemwallet.android.application.wallet_connect.WalletConnectJsonRpcRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionProposal
import com.gemwallet.android.application.wallet_connect.WalletConnectSessionRequest
import com.gemwallet.android.application.wallet_connect.WalletConnectValidation
import com.gemwallet.android.application.wallet_connect.WalletConnectVerifyContext
import com.gemwallet.android.domains.asset.aggregates.AssetInfoDataAggregate
import com.gemwallet.android.domains.asset.icon
import com.gemwallet.android.domains.confirm.FeeDetailsModel
import com.gemwallet.android.domains.confirm.FeeUIModel
import com.gemwallet.android.domains.wallet.aggregates.WalletDataAggregate
import com.gemwallet.android.ext.toGem
import com.gemwallet.android.model.AssetBalance
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.model.AssetPriceInfo
import com.gemwallet.android.model.AssetPriceValue
import com.gemwallet.android.model.Balance
import com.gemwallet.android.model.BuildInfo
import com.gemwallet.android.model.ChainAssetInfo
import com.gemwallet.android.model.Session
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetAssociation
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetMetaData
import com.wallet.core.primitives.AssetPrice
import com.wallet.core.primitives.BalanceMetadata
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Currency
import com.wallet.core.primitives.FeePriority
import com.wallet.core.primitives.NFTAssetId
import com.wallet.core.primitives.NFTCollectionId
import com.wallet.core.primitives.PerpetualId
import com.wallet.core.primitives.PerpetualProvider
import com.wallet.core.primitives.PlatformStore
import com.wallet.core.primitives.TransactionId
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletId
import uniffi.gemstone.FeeOption
import uniffi.gemstone.FeeUnitType
import uniffi.gemstone.GemAssetItemRow
import uniffi.gemstone.GemAssetItemTrailing
import uniffi.gemstone.GemFeeOptionItem
import uniffi.gemstone.GemFeeRateRows
import uniffi.gemstone.GemLocalizedText
import uniffi.gemstone.GemNumberUnit
import uniffi.gemstone.GemRowText
import uniffi.gemstone.GemValueTone
import uniffi.gemstone.GemWalletPlaceholder
import uniffi.gemstone.GemWalletRow
import uniffi.gemstone.GemWalletSubtitle
import uniffi.gemstone.feeAmount
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

fun mockAssetInfoDataAggregate(asset: Asset = mockAsset(), pinned: Boolean = false) = AssetInfoDataAggregate(
    asset = asset,
    row = GemAssetItemRow(
        icon = asset.id.icon(),
        title = asset.name,
        titleExtra = null,
        subtitle = null,
        subtitleExtra = null,
        trailing = GemAssetItemTrailing.Value(
            value = GemRowText(GemLocalizedText.Number(mockGemFormattedNumber(value = 1.0, unit = GemNumberUnit.Symbol(symbol = asset.symbol))), GemValueTone.PLAIN),
            extra = GemRowText(GemLocalizedText.Number(mockGemFormattedNumber(value = 1.0)), GemValueTone.NEUTRAL),
        ),
        masksBalance = true,
    ),
    hideBalance = false,
    pinned = pinned,
    balanceEnabled = true,
    accountAddress = mockAccount(chain = asset.id.chain).address,
)

fun mockBuildInfo(platformStore: PlatformStore = PlatformStore.GooglePlay) = BuildInfo(
    platformStore = platformStore,
    versionName = "1.0.0",
    versionCode = 1,
)

fun mockFeeDetailsModel(
    currentFee: FeeUIModel.FeeInfo = mockFeeInfo(),
    rows: GemFeeRateRows = mockGemFeeRateRows(unitType = FeeUnitType.GWEI, unitDecimals = 0u, supportsCustomFee = true, selectedTotal = BigInteger("2"), normalTotal = BigInteger("2")),
): FeeDetailsModel = FeeDetailsModel(
    currentFee = currentFee,
    rows = rows,
)

fun mockFeeInfo(
    amount: BigInteger = BigInteger("1000"),
    feeAsset: Asset = mockAsset(id = mockAssetId(chain = Chain.Ethereum), name = "Ethereum", symbol = "ETH", decimals = 18),
    price: Double? = null,
    additionalFees: List<Pair<FeeOption, BigInteger>> = emptyList(),
): FeeUIModel.FeeInfo {
    val formatted = { value: BigInteger -> feeAmount(feeAsset.toGem(), value, price, Currency.USD.toGem()) }
    return FeeUIModel.FeeInfo(
        amount = amount,
        feeAsset = feeAsset,
        price = price,
        currency = Currency.USD,
        priority = FeePriority.Normal,
        display = formatted(amount),
        additionalFees = additionalFees.map { (option, value) -> GemFeeOptionItem(option, value, formatted(value)) },
    )
}

fun mockSession(wallet: Wallet = mockWallet(), currency: Currency = Currency.USD) = Session(
    wallet = wallet,
    currency = currency,
)

fun mockWalletDataAggregate(row: GemWalletRow = mockGemWalletRow(id = "wallet-1", name = "Wallet", subtitle = GemWalletSubtitle.Multicoin, placeholder = GemWalletPlaceholder.Multicoin)): WalletDataAggregate =
    WalletDataAggregate(row = row, isCurrent = false)

fun mockWalletConnectSessionProposal(): WalletConnectSessionProposal {
    val metadata = mockApplicationMetadata()
    return WalletConnectSessionProposal(
        name = metadata.name,
        description = metadata.description,
        url = metadata.url,
        icons = listOf(metadata.icon),
        requiredNamespaces = emptyMap(),
        optionalNamespaces = emptyMap(),
        proposerPublicKey = "key",
        pairingTopic = "pairing",
        properties = null,
    )
}

fun mockWalletConnectSessionRequest(id: Long = 1, topic: String = "topic") = WalletConnectSessionRequest(
    topic = topic,
    chainId = "eip155:1",
    request = WalletConnectJsonRpcRequest(id = id, method = "personal_sign", params = "[]"),
)

fun mockWalletConnectVerifyContext(origin: String = "", validation: WalletConnectValidation = WalletConnectValidation.Valid, isScam: Boolean = false) = WalletConnectVerifyContext(
    origin = origin,
    validation = validation,
    isScam = isScam,
)

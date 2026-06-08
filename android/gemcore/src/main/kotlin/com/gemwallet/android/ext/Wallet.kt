package com.gemwallet.android.ext

import com.gemwallet.android.math.fromHex
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType

fun Wallet.getAccount(chain: Chain): Account? {
    return accounts.firstOrNull { it.chain == chain }
}

fun Wallet.getAccount(assetId: AssetId): Account? = getAccount(assetId.chain)

val WalletType.isViewOnly: Boolean get() = this == WalletType.View
val WalletType.canSign: Boolean get() = !isViewOnly

val Wallet.keystoreId: String
    get() = uniffi.gemstone.keystoreIdForWallet(id.id)

fun String.v4KeystorePasswordBytes(): ByteArray = fromHex()

fun String.legacyAndroidV3PasswordBytes(): ByteArray = fromHex()

val Wallet.hyperliquidAccount: Account?
    get() = accounts.firstOrNull {
        it.chain == Chain.Arbitrum || it.chain == Chain.HyperCore || it.chain == Chain.Hyperliquid
    }

val Wallet.hasPerpetualsSupport: Boolean
    get() = type == WalletType.Multicoin && hyperliquidAccount != null

val HypercoreUSDC: Asset = Asset(
    id = AssetId(chain = Chain.HyperCore, tokenId = "perpetual::USDC"),
    name = "USDC",
    symbol = "USDC",
    decimals = 6,
    type = AssetType.PERPETUAL,
)

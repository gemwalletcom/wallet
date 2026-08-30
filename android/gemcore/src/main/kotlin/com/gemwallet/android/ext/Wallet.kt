package com.gemwallet.android.ext

import com.gemwallet.android.domains.asset.defaultAssets
import com.gemwallet.android.math.fromHex
import com.wallet.core.primitives.Account
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.Wallet
import com.wallet.core.primitives.WalletType
import uniffi.gemstone.GemWalletType

fun Wallet.getAccount(chain: Chain): Account? {
    return accounts.firstOrNull { it.chain == chain }
}

fun Wallet.getAccount(assetId: AssetId): Account? = getAccount(assetId.chain)

val WalletType.isViewOnly: Boolean get() = this == WalletType.View

fun WalletType.toGem(): GemWalletType = when (this) {
    WalletType.Multicoin -> GemWalletType.MULTICOIN
    WalletType.Single -> GemWalletType.SINGLE
    WalletType.PrivateKey -> GemWalletType.PRIVATE_KEY
    WalletType.View -> GemWalletType.VIEW
}

val Wallet.hyperliquidAccount: Account?
    get() = accounts.firstOrNull {
        it.chain == Chain.Arbitrum || it.chain == Chain.HyperCore || it.chain == Chain.Hyperliquid
    }

val Wallet.hasPerpetualsSupport: Boolean
    get() = type == WalletType.Multicoin && hyperliquidAccount != null

val HypercoreUSDC: Asset = Chain.HyperCore.defaultAssets
    .first { it.type == AssetType.PERPETUAL }

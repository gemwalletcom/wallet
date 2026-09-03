package com.gemwallet.android.features.asset.viewmodels.details.models

import androidx.annotation.StringRes
import com.gemwallet.android.domains.price.ValueDirection
import com.gemwallet.android.model.AssetInfo
import com.gemwallet.android.ui.R
import com.wallet.core.primitives.Asset
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.AssetType
import com.wallet.core.primitives.BalanceMetadata
import com.wallet.core.primitives.Chain
import com.wallet.core.primitives.VerificationStatus
import com.wallet.core.primitives.WalletType

class AssetInfoUIModel(
    val assetInfo: AssetInfo,
    val name: String = "",
    val iconUrl: String = "",
    val priceValue: String = "0",
    val priceDayChanges: String = "0",
    val priceChangedType: ValueDirection = ValueDirection.Up,
    val tokenType: AssetType = AssetType.NATIVE,
    val accountInfoUIModel: AccountInfoUIModel = AccountInfoUIModel(),
    val isBuyEnabled: Boolean = false,
    val isSwapEnabled: Boolean = false,
    val swapPayAssetId: AssetId? = null,
    val swapReceiveAssetId: AssetId? = null,
    val explorerName: String = "",
    val explorerAddressUrl: String? = null,
    val explorerTokenUrl: String? = null,
    val verificationStatus: VerificationStatus? = null,
    val networkDestination: NetworkDestination? = null,
    val shareUrl: String = "",
    val updated: Long = System.currentTimeMillis(),
) {

    val asset: Asset get() = assetInfo.asset

    data class AccountInfoUIModel(
        val walletType: WalletType = WalletType.View,
        val totalBalance: String = "0",
        val totalFiat: String = "",
        val owner: String = "",
        val balances: List<BalanceUIModel> = emptyList(),
        val balanceMetadata: BalanceMetadata? = null,
    )

    sealed interface NetworkDestination {
        data class Asset(val assetId: AssetId) : NetworkDestination
        data class Assets(val chain: Chain) : NetworkDestination
    }

    data class BalanceUIModel(
        val type: BalanceViewType,
        val value: String = "0",
        val url: String? = null,
    )

    enum class BalanceViewType(@param:StringRes val label: Int) {
        Available(R.string.asset_balances_available),
        Stake(R.string.wallet_stake),
        PendingUnconfirmed(R.string.stake_pending),
        Reserved(R.string.asset_balances_reserved)
    }
}
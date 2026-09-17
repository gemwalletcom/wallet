package com.gemwallet.android.features.asset_select.viewmodels.localization

import androidx.annotation.StringRes
import com.gemwallet.android.ui.R
import uniffi.gemstone.GemSelectAssetTitle

@StringRes
internal fun GemSelectAssetTitle.stringRes(): Int = when (this) {
    GemSelectAssetTitle.SEND -> R.string.wallet_send
    GemSelectAssetTitle.RECEIVE -> R.string.wallet_receive
    GemSelectAssetTitle.RECEIVE_COLLECTION -> R.string.wallet_receive_collection
    GemSelectAssetTitle.BUY -> R.string.wallet_buy
    GemSelectAssetTitle.SWAP_PAY -> R.string.swap_you_pay
    GemSelectAssetTitle.SWAP_RECEIVE -> R.string.swap_you_receive
    GemSelectAssetTitle.MANAGE_TOKEN_LIST -> R.string.wallet_manage_token_list
    GemSelectAssetTitle.SELECT_ASSET -> R.string.assets_select_asset
    GemSelectAssetTitle.DEPOSIT -> R.string.wallet_deposit
    GemSelectAssetTitle.WITHDRAW -> R.string.wallet_withdraw
    GemSelectAssetTitle.SEARCH -> R.string.assets_select_asset
}

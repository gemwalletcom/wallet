package com.gemwallet.android.features.asset.presents.details

import com.gemwallet.android.model.ConfirmParams
import com.wallet.core.primitives.AssetId
import com.wallet.core.primitives.TransactionId

sealed interface AssetDetailsAction {
    sealed interface Navigation : AssetDetailsAction

    data object Refresh : AssetDetailsAction
    data object Pin : AssetDetailsAction
    data object Add : AssetDetailsAction
    data class TogglePriceAlert(val assetId: AssetId) : AssetDetailsAction

    data object Close : Navigation
    data class Transfer(val assetId: AssetId) : Navigation
    data class Receive(val assetId: AssetId) : Navigation
    data class Buy(val assetId: AssetId) : Navigation
    data class Swap(val fromAssetId: AssetId, val toAssetId: AssetId?) : Navigation
    data class OpenTransaction(val transactionId: TransactionId) : Navigation
    data class OpenChart(val assetId: AssetId) : Navigation
    data class OpenNetwork(val assetId: AssetId) : Navigation
    data class Stake(val assetId: AssetId) : Navigation
    data class OpenPriceAlerts(val assetId: AssetId) : Navigation
    data class Confirm(val params: ConfirmParams) : Navigation
}

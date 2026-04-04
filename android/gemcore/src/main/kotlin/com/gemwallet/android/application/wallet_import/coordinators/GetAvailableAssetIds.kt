package com.gemwallet.android.application.wallet_import.coordinators

fun interface GetAvailableAssetIds {
    suspend operator fun invoke(walletId: String): List<String>
}

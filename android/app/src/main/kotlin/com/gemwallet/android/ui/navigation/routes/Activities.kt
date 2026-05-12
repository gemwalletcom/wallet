package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.activities.presents.details.TransactionDetailsNavScreen
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.TransactionId
import kotlinx.serialization.Serializable

const val transactionsRoute = "transactions"

@Serializable
data class TransactionDetailsRoute(
    val transactionId: TransactionId
) : NavKey

fun EntryProviderScope<NavKey>.transactionDetailsScreen(
    onCancel: () -> Unit,
    onNft: (String) -> Unit,
) {
    entry<TransactionDetailsRoute>(
        metadata = { key ->
            routeArguments(RouteArgument.TransactionId to key.transactionId.identifier)
        },
    ) {
        TransactionDetailsNavScreen(onCancel = onCancel, onNft = onNft)
    }
}

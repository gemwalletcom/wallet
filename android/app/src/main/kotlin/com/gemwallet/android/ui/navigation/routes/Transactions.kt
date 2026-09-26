package com.gemwallet.android.ui.navigation.routes

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import com.gemwallet.android.features.transactions.presents.transaction.TransactionAction
import com.gemwallet.android.features.transactions.presents.transaction.TransactionScreen
import com.gemwallet.android.ui.models.navigation.RouteArgument
import com.gemwallet.android.ui.navigation.routeArguments
import com.wallet.core.primitives.TransactionId
import kotlinx.serialization.Serializable

const val TransactionsRoute = "transactions"

@Serializable
data class TransactionRoute(val transactionId: TransactionId) : NavKey

fun EntryProviderScope<NavKey>.transactionScreen(onAction: (TransactionAction.Navigation) -> Unit) {
    entry<TransactionRoute>(
        metadata = { key ->
            routeArguments(RouteArgument.TransactionId to key.transactionId.identifier)
        },
    ) {
        TransactionScreen(onAction = onAction)
    }
}
